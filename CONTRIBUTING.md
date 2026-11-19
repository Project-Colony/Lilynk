# Contributing to Lilynk

Building, testing and sending a change are in the
[developer docs](https://project-colony.github.io/Lilynk/docs/developers). Until the docs site is
published, the same pages are in `docs-site/content/docs/developers/`.

- Discuss a new feature or design with the maintainer before you open a pull request, on
  [Discord](https://discord.gg/tvFApEEVfj) or as a
  [GitHub issue](https://github.com/Project-Colony/Lilynk/issues). Bugs go on GitHub issues too.
  Report a security problem privately through the
  [advisory form](https://github.com/Project-Colony/Lilynk/security/advisories/new), not in an
  issue ([SECURITY.md](SECURITY.md)).
- Enable the hooks once per clone with `git config core.hooksPath scripts/git-hooks`
  ([what they run](https://project-colony.github.io/Lilynk/docs/developers/contributing#git-hooks)).
- Run the full pass before you push, plus the
  [checks for what you touched](https://project-colony.github.io/Lilynk/docs/developers/testing):

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test  --workspace --locked
  ```

- Open the pull request as a draft. The generated files to keep current are in
  [Contributing](https://project-colony.github.io/Lilynk/docs/developers/contributing#generated-files).
- Update the docs page that owns a fact in the same pull request
  ([where facts live](https://project-colony.github.io/Lilynk/docs/developers/contributing#docs)).
- Commits, comments, error messages and the changelog follow [docs/writing.md](docs/writing.md).

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
