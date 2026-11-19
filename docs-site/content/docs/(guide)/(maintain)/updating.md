---
title: Updating the Host
description: Update a source-built host by pulling its clone and building again, and turn the update check off.
---

Update the host by pulling the clone you built it from and building again. Lilynk publishes no
releases and no update feed yet, so the **Updates** card in the [web console](/docs/web-console)
never finds a newer host.

## Update by hand

| How you built it | Update |
|---|---|
| Ubuntu or Debian packages | As on [Ubuntu, step 2](/docs/ubuntu#2-install-the-host) |
| Fedora RPMs | As on [Fedora, step 2](/docs/fedora#2-install-the-host) |
| Arch, CachyOS or Omarchy packages | As on [Arch, step 2](/docs/arch#2-install-the-host) |
| Bazzite sysext | Build a new image, then `sudo lilynk-sysext update --from-file lilynk-X-x86-64.raw` |
| Steam Deck on-device build | `bash ~/lilynk/scripts/steamdeck/update.sh --pull` |
| NixOS (flake) | `nix flake update lilynk` in your flake directory, then rebuild |
| Windows setup | `git pull`, build and pack setup again, and run the new `lilynk-host-setup-<ver>.exe` over the old one |
| A bare `cargo build` | `git pull`, then the same `cargo build` |

### Restart after a Linux package update

Installing a new build restarts the running console, host and plugin runner for each signed-in
user: a package through pacman, apt or dnf, a NixOS switch, the Windows setup, `lilynk-sysext
update` and the Steam Deck script. A service you stopped stays stopped. A live stream drops when the
host restarts. A layered rpm-ostree install moves at the reboot instead.

A bare `cargo build` restarts by hand, console first:

```bash
systemctl --user restart lilynk-web
systemctl --user restart lilynk-host
systemctl --user try-restart lilynk-scripting
```

A host left running on a replaced binary fails every new KDE desktop session with
`KWin does not expose zkde_screencast_unstable_v1 to this client` until it restarts.

### Bazzite sysext: rollback and rebases

- `lilynk-sysext status` shows the installed build.
- Keep a build to go back to:

  ```sh
  sudo cp /var/lib/extensions/lilynk.raw ~/lilynk-known-good.raw   # before updating
  sudo lilynk-sysext install --from-file ~/lilynk-known-good.raw   # to go back
  ```

- After a Bazzite major rebase the old image refuses to load. Build an image for the new Fedora
  release and install it with `sudo lilynk-sysext update --from-file lilynk-X-x86-64.raw`.

## Turn the check off

Turn off **Check for updates** under **Host → System** (`LILYNK_UPDATE_CHECK=0` in
`host.env`). To keep the check but drop the button, turn off **Console updates** under
**Show advanced** (`LILYNK_UPDATE_APPLY=0`).

## Troubleshooting

Host problems: [Troubleshooting](/docs/troubleshooting). Clients:
[Install a Client → Keeping a client up to date](/docs/install-client#keeping-a-client-up-to-date).
