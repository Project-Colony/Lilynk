#!/bin/sh
# Behavioural gates for the guided installer, driven against the BINARY.
#
# scripts/install.sh is a stub that downloads lilynk-setup (design/installer-v2.md D1), so the
# behaviour lives in the binary. These gates are not in check-docs-drift.sh because that gate
# runs in a bun container with no cargo, and this needs a built installer.
#
# Two test seams: LILYNK_INSTALL_OS_RELEASE fakes the distro, LILYNK_INSTALL_ETC fakes the
# repo config. Stub binaries on PATH fake an installed host.
#
# Exit: 0 all cases held · 1 a case drifted · 2 the installer could not be built.
set -u
cd "$(dirname "$0")/../.." || exit 2

# CI passes a built binary; a developer running this by hand gets a debug build.
PF=${LILYNK_SETUP_BIN:-}
if [ -z "$PF" ]; then
    cargo build -q -p lilynk-setup || exit 2
    PF=target/debug/lilynk-setup
fi
[ -x "$PF" ] || { echo "::error::$PF is not executable"; exit 2; }

fail=0
osr=$(mktemp -d)
sw=$(mktemp -d)
trap 'rm -rf "$osr" "$sw"' EXIT

# ------------------------------------------------------------------ the detection matrix
# Faked os-release files through the real installer, nothing executed. This build has no package
# repo, so a fresh install names the family it detected, then stops with the build-from-source
# pointer; --uninstall still prints the family's removal. No output ever names an upstream feed.
# A fix to the installer adds its case here.
upstream_feed() {   # output -> 0 when it names an upstream package host
    case "$1" in *git.unom.io*|*flatpak.unom.io*) return 0 ;; esac
    return 1
}
installer_case() {   # name os-release-body expected-substring [extra args...]
    name=$1; printf '%b' "$2" > "$osr/$name"; want=$3; shift 3
    out=$(PATH="${pfpath:+$pfpath:}$PATH" LILYNK_INSTALL_OS_RELEASE="$osr/$name" \
          "$PF" --dry-run --yes --no-start "$@" 2>&1)
    case "$out" in *"$want"*) ;; *)
        echo "::error::lilynk-setup --dry-run $* on a fake $name os-release did not print '$want':"
        printf '%s\n' "$out" | sed 's/^/    /'
        fail=1 ;;
    esac
    if upstream_feed "$out"; then
        echo "::error::lilynk-setup --dry-run $* on a fake $name os-release names an upstream feed:"
        printf '%s\n' "$out" | grep -E 'git\.unom\.io|flatpak\.unom\.io' | sed 's/^/    /'
        fail=1
    fi
}
pfpath=
NOREPO="No package repo for"
installer_case debian   'ID=debian\nVERSION_ID=13\n'                    'Detected debian → apt'
installer_case debian2  'ID=debian\nVERSION_ID=13\n'                    "$NOREPO 'debian' yet"
installer_case ubuntu   'ID=ubuntu\nID_LIKE=debian\nVERSION_ID=26.04\n'  'Detected ubuntu → apt'
installer_case mint22   'ID=linuxmint\nID_LIKE="ubuntu debian"\nVERSION_ID=22.1\n' "can't host"
installer_case fedora   'ID=fedora\nVERSION_ID=44\n'                    'Detected fedora → dnf'
installer_case fedora2  'ID=fedora\nVERSION_ID=44\n'                    "$NOREPO 'fedora' yet"
installer_case arch     'ID=arch\n'                                    'Detected arch → pacman'
installer_case cachyos  'ID=cachyos\nID_LIKE="arch"\n'                  'Detected cachyos → pacman'
installer_case omarchy  'ID=omarchy\nID_LIKE=arch\nVERSION_ID=4.0.1\n'    'Detected omarchy → pacman'
installer_case bazzite  'ID=bazzite\nID_LIKE="fedora"\nVERSION_ID=43\n' 'Detected bazzite → sysext'
installer_case bazzite2 'ID=bazzite\nID_LIKE="fedora"\nVERSION_ID=43\n' "$NOREPO 'bazzite' yet"
installer_case nixos    'ID=nixos\n'                                   'docs/nixos'
# SteamOS says ID_LIKE=arch, so it has to beat the arch test or a Deck gets pacman commands its
# read-only /usr cannot run. The install is the on-device build, and a re-run must survive the
# ~/lilynk that the first one cloned.
installer_case steamos  'ID=steamos\nID_LIKE=arch\n'                    'bash ~/lilynk/scripts/steamdeck/install.sh'
installer_case steamos2 'ID=steamos\nID_LIKE=arch\n'                    '[ -d ~/lilynk/.git ] || git clone'
# A Deck clones main: the origin has no release branch yet, with or without --channel.
installer_case steamos3 'ID=steamos\nID_LIKE=arch\n'                    'git clone https://github.com/Project-Colony/Lilynk ~/lilynk'
installer_case steamos-canary 'ID=steamos\nID_LIKE=arch\n'              'git clone https://github.com/Project-Colony/Lilynk ~/lilynk' --channel canary
installer_case holoiso  'ID=holoiso\nID_LIKE="steamos arch"\n'          'scripts/steamdeck/install.sh'
installer_case gentoo   'ID=gentoo\n'                                  'build-from-source'
# A client install would add the flatpak remote, or the family's repo, so it stops too.
installer_case gentoo-client 'ID=gentoo\n'                             "$NOREPO 'gentoo' yet" --client
installer_case arch-client   'ID=arch\n'                               "$NOREPO 'arch' yet" --client
installer_case debian-rm 'ID=debian\nVERSION_ID=13\n'                   'sources.list.d/lilynk.list' --uninstall
installer_case fedora-rm 'ID=fedora\nVERSION_ID=44\n'                   'yum.repos.d/lilynk.repo' --uninstall
installer_case arch-rm   'ID=arch\n'                                   '/etc/pacman.conf' --uninstall
installer_case omarchy-rm 'ID=omarchy\nID_LIKE=arch\n'                  'lilynk-omarchy remove' --uninstall
installer_case bazzite-rm 'ID=bazzite\nID_LIKE="fedora"\nVERSION_ID=43\n' 'lilynk-sysext remove' --uninstall
# Nothing on SteamOS is a package, so --uninstall must say so instead of claiming a removal.
installer_case steamos-rm 'ID=steamos\nID_LIKE=arch\n'                  'no uninstall script' --uninstall

LILYNK_SETUP_BIN="$PF" sh scripts/ci/check-install-defaults.sh || fail=1

# --------------------------------------------------------------- an installed box, wiring only
# All three binaries on PATH and no repo config: the box goes on to the setup phases and installs
# nothing. A repo config left on the box (an upstream install) stops the run instead, so no
# channel switch rewrites a repo. Two fakes make this reachable under --dry-run: stub binaries on
# PATH, and LILYNK_INSTALL_ETC pointing at the repo config the box is supposedly on.
mkdir -p "$sw/bin"
for b in lilynk-host lilynk-web-server lilynk-scripting; do
    printf '#!/bin/sh\necho 0.0.0-test\n' > "$sw/bin/$b"; chmod +x "$sw/bin/$b"
done
pfpath="$sw/bin"
WIRED="skipping the install"
installer_case debian-wired 'ID=debian\nVERSION_ID=13\n'                 "$WIRED"
installer_case arch-wired   'ID=arch\n'                                "$WIRED"
# Omarchy: the run hands off to `lilynk-omarchy setup` rather than do a second, weaker version
# of the same wiring.
installer_case omarchy-wired 'ID=omarchy\nID_LIKE=arch\nVERSION_ID=4.0.1\n' 'lilynk-omarchy setup'
pfpath=
switch_case() {   # name os-release-body config-path config-body expected-substring [extra args...]
    name=$1; printf '%b' "$2" > "$osr/$name"
    mkdir -p "$sw/$name/$(dirname "$3")"; printf '%b' "$4" > "$sw/$name/$3"; want=$5; shift 5
    out=$(PATH="$sw/bin:$PATH" LILYNK_INSTALL_OS_RELEASE="$osr/$name" LILYNK_INSTALL_ETC="$sw/$name" \
          "$PF" --dry-run --yes --no-start "$@" 2>&1)
    case "$out" in *"$want"*) ;; *)
        echo "::error::lilynk-setup --dry-run $* on an installed $name box did not print '$want':"
        printf '%s\n' "$out" | sed 's/^/    /'
        fail=1 ;;
    esac
    if upstream_feed "$out"; then
        echo "::error::lilynk-setup --dry-run $* on an installed $name box names an upstream feed:"
        printf '%s\n' "$out" | grep -E 'git\.unom\.io|flatpak\.unom\.io' | sed 's/^/    /'
        fail=1
    fi
}
DEB13='ID=debian\nVERSION_ID=13\n'
switch_case apt-repo "$DEB13" etc/apt/sources.list.d/lilynk.list \
    'deb [x] https://git.unom.io/api/packages/unom/debian stable main\n' "$NOREPO 'debian' yet" --channel canary
switch_case dnf-repo 'ID=fedora\nVERSION_ID=44\n' etc/yum.repos.d/lilynk.repo \
    'baseurl=https://git.unom.io/api/packages/unom/rpm/fedora-44\n' "$NOREPO 'fedora' yet" --channel canary
switch_case pac-repo 'ID=arch\n' etc/pacman.conf '[lilynk-canary]\nServer = x\n' "$NOREPO 'arch' yet" --channel stable
# A sysext has no repo file: an installed one, even from a feed's channel, is wiring only.
switch_case sys-wired 'ID=bazzite\nID_LIKE="fedora"\nVERSION_ID=43\n' etc/lilynk-sysext.conf 'CHANNEL=canary\n' \
    "$WIRED" --channel stable

exit $fail
