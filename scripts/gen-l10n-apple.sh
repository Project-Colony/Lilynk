#!/usr/bin/env bash
# Write the Apple client's String Catalog and L10n.swift from l10n/messages into
# clients/apple/Sources/LilynkShared/Generated, which git ignores. Run it before
# `swift build`, `swift test` or an Xcode build; scripts/build-xcframework.sh runs it too.
# `--root` names this checkout's messages: a cargo target dir shared with another worktree
# otherwise hands back its binary, which reads its own.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo run -q -p ly-l10n -- apple --root "$PWD/l10n" \
    --out clients/apple/Sources/LilynkShared/Generated --parts common,client,apple,host
