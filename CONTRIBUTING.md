# Contributing to Punktfunk

Thanks for your interest in contributing! Building, testing and sending a change are in the
[developer docs](https://docs.punktfunk.unom.io/docs/developers). The short version:

- **Discuss features first.** Talk a new feature or design through with the maintainer, on
  [Discord](https://discord.gg/wzEGg9y45z) or as an idea on
  [improve unom](https://improve.unom.io/p/punktfunk), before you open an issue or a pull
  request. Bugs go on improve unom too. Security reports go to security@punktfunk.com
  ([SECURITY.md](SECURITY.md)).
- **Enable the hooks** once per clone: `git config core.hooksPath scripts/git-hooks`
  ([what they run](https://docs.punktfunk.unom.io/docs/developers/contributing#git-hooks)).
- **Run the full pass** before you push, plus the
  [checks for what you touched](https://docs.punktfunk.unom.io/docs/developers/testing):

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test  --workspace --locked
  ```

- **Open the pull request as a draft** (`WIP:` title prefix). CI lanes, the `ci:*` labels and the
  generated files CI checks:
  [Contributing](https://docs.punktfunk.unom.io/docs/developers/contributing#open-a-pull-request).
- **Update the docs page that owns a fact** in the same pull request:
  [where facts live](https://docs.punktfunk.unom.io/docs/developers/contributing#docs).
- **Commits, comments, error messages and the changelog** follow
  [docs/writing.md](docs/writing.md).

## Licensing of contributions (inbound = outbound)

Lilynk is licensed under **GPL-3.0-or-later**. It is a fork of punktfunk, which is licensed under
MIT OR Apache-2.0; the [License section of the README](README.md#license) explains how the two
apply.

By opening a pull request you agree to license your contribution under GPL-3.0-or-later. You keep
the copyright to your contributions.

### Code from other projects

Code copied from another project must be under a license compatible with GPL-3.0-or-later, for
example MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0, LGPL-2.1-or-later or GPL-3.0. Keep its copyright
and license notices, and name the source project and file in the commit message. Code under
GPL-2.0-only and code with no license cannot be used.

New dependencies follow the same rule. `about.toml` holds the accepted-license list: add a new SPDX
id there only after checking that license, and regenerate the attribution file with
`scripts/gen-third-party-notices.sh` when the dependency tree changes.
