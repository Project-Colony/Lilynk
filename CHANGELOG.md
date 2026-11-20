# Changelog

Protocol, ABI, driver and embedder changes, one card per stable release, newest first. What
changed for people who use Lilynk goes in `docs/releases/vX.Y.Z.md`; `docs/writing.md` §2 has the
rules for both.

Lilynk forks punktfunk's main at commit `f1e09f58c`, 386 commits after the v0.47.0 tag.
punktfunk's changelog and release notes up to the v0.47.0 tag are kept unchanged in
[docs/upstream/](docs/upstream/CHANGELOG.md).

## Unreleased

The first Lilynk changes after the fork. Wire protocol `lilynk/2` (ALPN `lyk2`); C ABI 76.

### Breaking

- The wire protocol is renamed: mDNS `_lilynk._udp`, ALPN `lyk2`, the pairing, unlock and
  exporter labels, `X-Lilynk-*` headers and `lilynk://` links. Lilynk and punktfunk clients and
  hosts do not connect to each other.
- Binaries, packages and units are `lilynk-*`, crates `lilynk-*` and `ly-*`, environment variables
  `LILYNK_*`, application ids `org.projectcolony.lilynk*`, and configuration lives in
  `~/.config/lilynk`.
- C ABI 76: the header is `include/lilynk_core.h` and its symbols are `lilynk_*`.
- Update checks are off and no update feed, plugin catalog or package repository is configured.
