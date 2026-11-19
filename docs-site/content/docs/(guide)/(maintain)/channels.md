---
title: Release Channels
description: Lilynk has no release channels yet. A source build follows the branch of the clone it was built from.
---

A Lilynk build follows the branch of the clone it was built from. Lilynk publishes no packages,
releases or update feeds yet, so there is no stable or canary channel to subscribe to. The
repository has one branch, `main`.

| Build | Its clone |
|---|---|
| Linux packages, Windows setup, clients | Wherever you cloned the repository |
| SteamOS on-device build | `~/lilynk` |
| NixOS | The flake input, `git+https://github.com/Project-Colony/Lilynk` |

To pin a version, check out a commit in the clone and build it; `git switch main` goes back. On a
Steam Deck, build with `bash ~/lilynk/scripts/steamdeck/update.sh`, without `--pull`. On NixOS, add
`?rev=<commit>` to the flake input. To update, see [Updating](/docs/updating).
