#!/usr/bin/env bash
# Build the lilynk-web .deb — the management web console (Nitro SSR on bun + React).
#
# Runtime is BUN: the console is built with Nitro's `bun` preset + a custom Bun.serve entry that
# serves HTTPS (HTTP/1.1 over TLS) with the host's identity cert (web/nitro-entry/bun-https.mjs). It
# runs on the vendored bun from lilynk-bun (build-bun-deb.sh), pinned to this exact version. The
# host's lilynk-host .deb Recommends this, so a default `apt install lilynk-host` pulls the
# console too; it is auto-wired to the host's mgmt token + identity cert via the systemd --user
# units (no env editing on a packaged install).
#
# Usage: VERSION=0.0.1~ci42.gdeadbee [DEB_ARCH=amd64] bash packaging/debian/build-web-deb.sh
# Output: dist/lilynk-web_<version>_<arch>.deb
set -euo pipefail

VERSION="${VERSION:?set VERSION (e.g. 0.0.1 or 0.0.1~ci42.gdeadbee)}"
PKG="lilynk-web"
ROOTDIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOTDIR"

# Per-arch, like the lilynk-bun it depends on.
DEB_ARCH="${DEB_ARCH:-$(dpkg --print-architecture)}"

# Build the console if not already built (.output is gitignored — CI builds it each run).
if [ ! -f web/.output/server/index.mjs ]; then
  echo "==> building web console"
  (cd web && bun install --frozen-lockfile && bun run build)
fi
# The build MUST be the bun preset (our Bun.serve TLS entry) — node can't run Bun.serve.
if ! grep -rq 'Bun\.serve' web/.output/server/index.mjs 2>/dev/null; then
  echo "ERROR: web/.output has no Bun.serve — wrong nitro preset (need 'bun' + the custom entry)" >&2
  exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
SHAREDIR="$STAGE/usr/share/$PKG"
DOCDIR="$STAGE/usr/share/doc/$PKG"

# --- file layout -------------------------------------------------------------
mkdir -p "$SHAREDIR/.output"
cp -r web/.output/server "$SHAREDIR/.output/server"
cp -r web/.output/public "$SHAREDIR/.output/public"
# Stable PATH-independent ExecStart wrapper.
install -d "$STAGE/usr/bin"
cat > "$STAGE/usr/bin/lilynk-web-server" <<'WRAP'
#!/bin/sh
# The console runs on lilynk-bun's private bun (Bun.serve TLS), never one from PATH.
exec /usr/lib/lilynk-bun/bun /usr/share/lilynk-web/.output/server/index.mjs "$@"
WRAP
chmod 0755 "$STAGE/usr/bin/lilynk-web-server"
install -Dm0644 scripts/lilynk-web.service      "$STAGE/usr/lib/systemd/user/lilynk-web.service"
install -Dm0644 scripts/lilynk-web-init.service "$STAGE/usr/lib/systemd/user/lilynk-web-init.service"
install -Dm0755 scripts/web-init.sh                "$SHAREDIR/web-init.sh"
install -Dm0644 web/web.env.example                "$SHAREDIR/web.env.example"
install -Dm0644 LICENSE                             "$DOCDIR/LICENSE"
install -Dm0644 LICENSE-MIT                         "$DOCDIR/LICENSE-MIT"
install -Dm0644 LICENSE-APACHE                      "$DOCDIR/LICENSE-APACHE"
install -Dm0644 web/README.md                       "$DOCDIR/README.md"

cat > "$DOCDIR/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: lilynk
Source: https://github.com/Project-Colony/Lilynk

Files: *
Copyright: punktfunk contributors
License: GPL-3+
Comment: Based on punktfunk, which is licensed under MIT or Apache-2.0, at your
 option. The punktfunk code stays available under those terms. Full texts in
 /usr/share/doc/$PKG/LICENSE-MIT and /usr/share/doc/$PKG/LICENSE-APACHE.

License: GPL-3+
 This program is free software: you can redistribute it and/or modify it under
 the terms of the GNU General Public License as published by the Free Software
 Foundation, either version 3 of the License, or (at your option) any later
 version.
 .
 On Debian systems, the complete text of the GNU General Public License version
 3 can be found in /usr/share/common-licenses/GPL-3. A copy is also in
 /usr/share/doc/$PKG/LICENSE.
EOF
printf '%s (%s) stable; urgency=medium\n\n  * Automated build %s.\n\n -- unom <packages@unom.io>  %s\n' \
  "$PKG" "$VERSION" "$VERSION" "$(date -uR 2>/dev/null || echo 'Thu, 01 Jan 1970 00:00:00 +0000')" \
  | gzip -9n > "$DOCDIR/changelog.Debian.gz"

INSTALLED_KB="$(du -k -s "$STAGE" | cut -f1)"

install -d "$STAGE/DEBIAN"
cat > "$STAGE/DEBIAN/control" <<EOF
Package: $PKG
Version: $VERSION
Architecture: $DEB_ARCH
Maintainer: unom <packages@unom.io>
Installed-Size: $INSTALLED_KB
Section: net
Priority: optional
Depends: lilynk-bun (= $VERSION)
Homepage: https://github.com/Project-Colony/Lilynk
Description: lilynk management web console (Nitro SSR on bun + React)
 The browser console for a lilynk streaming host: status, paired devices, and the
 SPAKE2 PIN pairing flow every client needs. Runs as a systemd --user service on port
 47992 over HTTPS (HTTP/1.1 over TLS, with the host's own identity cert), login-gated (a
 password generated on first start), proxying the host's loopback HTTPS management API
 with a bearer token injected server-side (never sent to the browser). Runs on the bun
 from lilynk-bun (no system nodejs/bun dependency).
 .
 Auto-wired to the host on a packaged install: it sources the host's
 ~/.config/lilynk/mgmt-token and a generated login password — no env editing. Enable
 the systemd user service lilynk-web; read the login password out of web-password once,
 before the console replaces it with a salted argon2id hash on your first sign-in.
EOF

cat > "$STAGE/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    echo "lilynk-web installed. Enable it for your user:"
    echo "    systemctl --user enable --now lilynk-web"
    echo "A login password is generated on first start. Read it once, before you sign in:"
    echo "    sed -n 's/^LILYNK_UI_PASSWORD=//p' ~/.config/lilynk/web-password"
    echo "After that the console keeps only a salted hash, so a forgotten password is reset:"
    echo "put a LILYNK_UI_PASSWORD=<your-password> line in that file, then"
    echo "    systemctl --user restart lilynk-web"
    echo "Then open https://<host-ip>:47992 (self-signed host cert — trust it once)"
fi
exit 0
EOF
chmod 0755 "$STAGE/DEBIAN/postinst"

mkdir -p dist
OUT="dist/${PKG}_${VERSION}_${DEB_ARCH}.deb"
dpkg-deb --root-owner-group --build "$STAGE" "$OUT" >/dev/null
echo "built $OUT"
dpkg-deb -I "$OUT" | sed -n 's/^/  /p' | grep -E 'Version|Installed-Size|Depends' || true
