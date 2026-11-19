#!/bin/sh
# punktfunk guided installer — PREVIEW.
#
#   cargo build --release -p lilynk-setup
#   LILYNK_SETUP_BIN=target/release/lilynk-setup sh scripts/install.sh
#
# This is a stub. It checks that this is a Linux box it has a build for, then execs a
# `lilynk-setup` binary with every argument and every LILYNK_INSTALL_* variable untouched.
# The binary is LILYNK_SETUP_BIN, a local file, or a download from LILYNK_SETUP_REGISTRY,
# checked against its sha256. No registry is published for this build, so there is no default.
# The installer itself — distro detection, the guided screen, the plan it runs, uninstall,
# channel switching — lives in that binary (crates/install/lilynk-setup,
# design/installer-v2.md).
#
# THE STUB NEVER GROWS A FLAG. The binary owns the interface, so a copy of this file cached
# years ago keeps working against a newer installer. Anything that looks like a new option
# belongs in the binary, which is what parses it.
#
# The sha256 comes from the same host as the binary, so it catches a truncated or corrupted
# download, not a hostile registry: TLS to that registry is the trust anchor.
#
# Exit codes: 0 done · 1 unsupported system / a step failed · 2 bad usage.
set -u

DOCS=https://docs.punktfunk.unom.io/docs
REGISTRY=${LILYNK_SETUP_REGISTRY:-}
CHANNEL_DIR=${LILYNK_SETUP_VERSION:-latest}

die() { printf '\033[1;31m  xx\033[0m %s\n' "$*" >&2; exit 1; }

# --help is answered here so that "download it and read it first" stays meaningful: you can see
# what this file does without running it. The option list is the binary's and is printed by the
# binary, because only it knows what it accepts.
for arg in "$@"; do
    case "$arg" in
        -h|--help)
            cat <<EOF
punktfunk guided installer (preview)

This script runs the lilynk-setup binary. It takes no options of its own — everything you
pass is handed to the installer untouched, as is every LILYNK_INSTALL_* variable in the
environment.

  sh install.sh --yes            install unattended, taking every default
  sh install.sh --dry-run        print every command it would run, change nothing
  sh install.sh --uninstall      remove the packages and the repo
  sh install.sh --demo PRESET    walk the whole flow against a canned box, changing nothing

Those four are the ones worth knowing before you start. The installer accepts more — component
and channel selection, the per-option environment twins, the Omarchy hand-off — and prints the
authoritative list itself: $DOCS/install, or "lilynk-setup --help" once it is on the box.

No installer download is published for this build. Build one and point the script at it:

  cargo build --release -p lilynk-setup
  LILYNK_SETUP_BIN=target/release/lilynk-setup sh install.sh …

LILYNK_SETUP_REGISTRY=<url> downloads it from a registry of your own instead, checked
against the sha256 published next to it.

Docs: $DOCS/install
EOF
            exit 0
            ;;
    esac
done

cat <<EOF

  punktfunk guided installer — PREVIEW
  The per-distro pages stay the documented path; this automates them. Docs: $DOCS/install

EOF

[ "$(uname -s)" = Linux ] || die "this installer is for Linux hosts — Windows: $DOCS/windows-host"

case "$(uname -m)" in
    x86_64|amd64)   ARCH=x86_64 ;;
    aarch64|arm64)  ARCH=aarch64 ;;
    *) die "no lilynk-setup build for $(uname -m) — build from source: $DOCS/build-from-source" ;;
esac

# LILYNK_SETUP_BIN skips the download entirely. installer-smoke points it at the binary built
# from the pull request's own tree, so CI tests the checked-out code and never the published one.
if [ -n "${LILYNK_SETUP_BIN:-}" ]; then
    [ -x "$LILYNK_SETUP_BIN" ] || die "LILYNK_SETUP_BIN=$LILYNK_SETUP_BIN is not executable"
    exec "$LILYNK_SETUP_BIN" "$@"
fi

[ -n "$REGISTRY" ] || die "no installer download is published for this build — build it with 'cargo build --release -p lilynk-setup', then run LILYNK_SETUP_BIN=target/release/lilynk-setup sh install.sh"

command -v curl >/dev/null 2>&1 || die "curl is required (install it with your package manager first)"

TMP=$(mktemp -d) || die "could not create a temporary directory"
trap 'rm -rf "$TMP"' EXIT INT TERM

NAME=lilynk-setup_$ARCH
URL=$REGISTRY/$CHANNEL_DIR/$NAME

curl -fsSL -o "$TMP/$NAME" "$URL" \
    || die "could not download the installer from $URL — check your connection, or follow the page for your distro: $DOCS/install"
curl -fsSL -o "$TMP/$NAME.sha256" "$URL.sha256" \
    || die "could not download the installer's checksum from $URL.sha256 — $DOCS/install"

# No checksum tool means the download cannot be verified, and an unverified binary does not get
# executed. Every distro this installer supports ships coreutils.
want=$(tr -d ' \t\n\r' < "$TMP/$NAME.sha256")
if command -v sha256sum >/dev/null 2>&1; then
    got=$(sha256sum "$TMP/$NAME" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
    got=$(shasum -a 256 "$TMP/$NAME" | awk '{print $1}')
elif command -v openssl >/dev/null 2>&1; then
    got=$(openssl dgst -sha256 "$TMP/$NAME" | awk '{print $NF}')
else
    die "no sha256 tool (sha256sum, shasum or openssl) — cannot verify the download: $DOCS/install"
fi
[ -n "$want" ] || die "the published checksum for $NAME is empty — $DOCS/install"
[ "$got" = "$want" ] || die "checksum mismatch for $NAME (got $got, expected $want) — the download is corrupt; retry, or follow $DOCS/install"

chmod +x "$TMP/$NAME" || die "could not make the installer executable"

# The trap cannot fire across exec, so the temporary directory outlives this shell, and the
# installer runs from it.
exec "$TMP/$NAME" "$@"
