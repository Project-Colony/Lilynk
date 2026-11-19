---
title: Running as a Service
description: Start the host at boot — on a desktop you log into, or on a headless always-on machine.
---

Run the host as a service so it is always there: from login on a desktop, from boot on a headless
box.

## What the unit starts

The service runs `lilynk-host serve`: the Lilynk plane and the management API. Two more
planes stay off until you turn them on under **Host** in the
[web console](/docs/web-console), or in `~/.config/lilynk/host.env`:

| Plane | Setting | `host.env` | Firewall |
|---|---|---|---|
| Stock [Moonlight](/docs/moonlight) (GameStream) | **GameStream** | `LILYNK_GAMESTREAM=1` | open the `lilynk-gamestream` service |
| [Browser client](/docs/browser-client) (preview) | **Browser streaming** | `LILYNK_WEBTRANSPORT=1` | UDP 9778, in `lilynk-native` (`LILYNK_WEBTRANSPORT_PORT` moves it) |

Both take effect after a host restart. GameStream pairs over plain HTTP with weaker encryption, so
turn it on only on a LAN you trust ([Security](/docs/security)).
`systemctl --user cat lilynk-host` shows the unit in effect, drop-ins included.

## A desktop you log into [#a-a-desktop-you-log-into]

The packages ship a systemd user unit. Enable it once and the host starts at every login:

```sh
systemctl --user daemon-reload             # needed on the Bazzite sysext, harmless elsewhere
systemctl --user enable --now lilynk-host
systemctl --user status lilynk-host
```

The host finds your live session by itself, so it needs no `host.env` and no
`systemctl --user import-environment`. For settings, use the console or copy a template to
`~/.config/lilynk/host.env` — they are in `/usr/share/lilynk/` (`/usr/share/lilynk-host/`
on Ubuntu); every knob is in [Configuration](/docs/configuration). A line in `host.env` locks
that setting in the console until you delete it. Built from source?
[Build from source](/docs/developers/build-from-source) installs the unit.

### Restart the host with your desktop

Without this drop-in, restarting Plasma or GNOME (a crash, logging out and back in) leaves the host
answering but failing every session at capture. The drop-in makes a desktop restart a host restart:

```sh
mkdir -p ~/.config/systemd/user/lilynk-host.service.d
# /usr/share/lilynk-host/ on Ubuntu
cp /usr/share/lilynk/lilynk-host-desktop-session.conf \
   ~/.config/systemd/user/lilynk-host.service.d/desktop-session.conf
systemctl --user daemon-reload
systemctl --user reenable lilynk-host
systemctl --user restart lilynk-host
```

On NixOS set `services.lilynk.host.desktopSession = true;` instead.

Skip it on a headless box (below). Sway, and Hyprland started without uwsm (the Universal Wayland
Session Manager), never reach systemd's `graphical-session.target`, so the drop-in does nothing
there. To restart the host with those sessions, disable the unit and start it from the compositor
config instead: `exec systemctl --user start lilynk-host` (sway) or
`exec-once = systemctl --user start lilynk-host` (Hyprland). `niri-session` and `start-umbriel`
do reach it, so the drop-in works under [niri](/docs/niri) and [Umbriel](/docs/umbriel).

## A headless, always-on host

A box with no monitor and no login needs the host to start without a login, and a desktop session
that comes up at boot.

1. Let user services start at boot (the guided installer's `--linger` does the same):

   ```sh
   sudo loginctl enable-linger "$USER"
   ```

2. Bring up a session at boot:
   - GNOME: [GNOME → Headless session](/docs/gnome#headless-session).
   - KDE Plasma: [KDE → Headless session](/docs/kde#headless-session).
   - Steam / gamescope, Bazzite: nothing to set up — the host starts a session per client
     ([gamescope](/docs/gamescope)).
3. Enable the host unit as above, and reboot.

## Windows

The installer registers the host as the **`LilynkHost`** service: `LocalSystem`, started at
boot, so it captures the lock screen and UAC prompts with nobody logged in. Keep a host with that
much reach on a network you trust ([Security](/docs/security)).

Manage it from an elevated prompt:

```powershell
lilynk-host service status
lilynk-host service restart
```

Every `service` subcommand is in [Host CLI](/docs/host-cli#service-windows). The planes in
[What the unit starts](#what-the-unit-starts) work the same way; `host.env` is
`%ProgramData%\lilynk\host.env`. The installer opens the firewall on Private and Domain networks
only; a LAN Windows calls Public needs [one more step](/docs/troubleshooting#windows-firewall).

## Verifying

After a reboot, from another machine:

```sh
lilynk reachable 192.168.1.50   # exit 0 = the host answered, 2 = it didn't
lilynk hosts list --probe       # every saved host, online or offline
```

`lilynk` ships with the Linux client packages and the Windows client. Any client that lists the
host works too. No answer? Read `journalctl --user -u lilynk-host` on the host, or
`lilynk-host service status` on Windows.

## GPU scheduling priority

[PyroWave](/docs/pyrowave) encodes on the same GPU cores as your game, and on NVIDIA the host
converts every frame there before the video engine encodes it. So the host asks the driver to run
that work first. The request needs `CAP_SYS_NICE`, which every Linux install grants to a small
helper, `lilynk-encode-worker` — never to `lilynk-host`.

**Never give `lilynk-host` a capability** — not with `setcap`, a systemd `AmbientCapabilities=`
line or a NixOS `security.wrappers` entry. KWin can't identify a host that holds one, and every KDE
session fails at capture with:

```
KWin does not expose zkde_screencast_unstable_v1 to this client
```

Check both binaries:

```sh
getcap /usr/bin/lilynk-host              # should print nothing
getcap /usr/bin/lilynk-encode-worker     # /usr/bin/lilynk-encode-worker cap_sys_nice=ep
sudo setcap -r /usr/bin/lilynk-host      # clears it; then restart the host
```

On the Bazzite image `/usr` is read-only: run `sudo lilynk-sysext update` instead. On NixOS the
module wraps the worker for you.

Without the grant the stream still runs, at normal priority, but a GPU-bound game adds milliseconds
to every frame. The host log then says `VkBridge queue at default GPU priority`. `PYROWAVE_QUEUE_PRIORITY=off` and `LILYNK_ENCODE_WORKER=off` are the opt-outs —
[Configuration](/docs/configuration#advanced-performance-tuning).

## Stopping and removing

```sh
systemctl --user stop lilynk-host        # Linux, until next login
systemctl --user disable --now lilynk-host
lilynk-host service stop                 # Windows, elevated
```

`lilynk-host service uninstall` removes the Windows service and its firewall rules; the rest of
the install goes through **Settings → Apps**. A Linux package update restarts the running service
for you — [Updating](/docs/updating#restart-after-a-linux-package-update). Your config
and pairings survive all of this; [Uninstall](/docs/uninstall) clears them.
