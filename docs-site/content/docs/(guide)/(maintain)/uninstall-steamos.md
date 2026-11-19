---
title: Uninstalling a SteamOS Host
description: Remove the host you built on a Steam Deck or another SteamOS box — the services, the build container, the files under your home and the root-owned tuning.
---

Remove a Lilynk host you built on a Steam Deck or another SteamOS box, including the files that
would otherwise survive every SteamOS update.

## SteamOS / Steam Deck host (on-device build)

There is no uninstall script. `sh install.sh --uninstall` only stops the services; do the rest by
hand, in this order.

1. Stop and remove the user services:

   ```sh
   systemctl --user disable --now lilynk-host lilynk-web \
     lilynk-scripting lilynk-rebuild-check
   rm -f ~/.config/systemd/user/lilynk-*.service
   rm -rf ~/.config/systemd/user/lilynk-*.service.d
   systemctl --user daemon-reload
   sudo loginctl disable-linger "$USER"      # only if nothing else on the Deck needs it
   ```

2. Remove the build container and the files under your home:

   ```sh
   distrobox rm -f pf2                       # the build container (~1 GB)
   rm -f  ~/.local/bin/lilynk-scripting ~/.local/bin/lilynk-gamescope
   rm -rf ~/.local/lib/lilynk-scripting ~/.local/share/lilynk-scripting
   rm -f  ~/.local/share/lilynk/gamescope.stamp
   rm -f  ~/.local/share/applications/org.projectcolony.lilynk.host.desktop
   rm -f  ~/.local/bin/lilynk-tray ~/.config/autostart/org.projectcolony.lilynk.tray.desktop
   rm -f  ~/.local/share/applications/org.projectcolony.lilynk.starthost.desktop
   rm -f  ~/.local/share/icons/hicolor/*/apps/lilynk-tray*.png
   rm -rf ~/lilynk                        # the source checkout and its build
   ```

   The container shares your home, so `~/.cargo`, `~/.rustup` and `~/.bun` are there too. Delete
   them only if nothing else of yours uses Rust or bun.

3. If you set up [Reachable without logging in](/docs/profiles#on-linux), turn it off under
   **Profiles** first, then remove it. The last line deletes every seat's home and the shared
   games folder:

   ```sh
   sudo systemctl disable --now lilynk-web-door lilynk-door lilynk-seats \
     var-lib-lilynk.mount 'var-lib-lilynk\x2dseats.mount'
   sudo rm -rf /opt/lilynk /etc/systemd/system/lilynk-seat@.service.d
   sudo rm -f /etc/systemd/system/lilynk-{seats,seat@,door,web-door,door-on@,door-off@}.service \
     /etc/systemd/system/var-lib-lilynk*.mount \
     /etc/systemd/system/user@.service.d/50-lilynk-seats.conf \
     /etc/systemd/user/lilynk-scripting.service /etc/pam.d/lilynk-seat \
     /etc/polkit-1/rules.d/49-lilynk-door.rules \
     /etc/udev/rules.d/65-lilynk-seats.rules /etc/tmpfiles.d/lilynk-seats.conf
   sudo rm -rf /home/.steamos/offload/var/lib/lilynk /home/.steamos/offload/var/lib/lilynk-seats
   ```

   Each seat's account stays: `getent passwd | grep lilynk-seat=` lists them for
   `sudo userdel`, and `sudo userdel lilynk` removes the door's. Then drop the groups it used:
   `sudo groupdel lilynk-door; sudo groupdel lilynk-update; sudo groupdel lilynk-games`.

4. Remove the root-owned tuning. Don't skip it: the keep-list entry carries these files through
   every SteamOS update.

   ```sh
   sudo rm -f /etc/atomic-update.conf.d/lilynk.conf \
              /etc/udev/rules.d/60-lilynk.rules \
              /etc/modules-load.d/lilynk.conf \
              /etc/sysctl.d/99-lilynk-net.conf \
              /etc/NetworkManager/dispatcher.d/90-lilynk-frame-adapter \
              /etc/systemd/system/user@.service.d/50-lilynk-nice.conf
   sudo udevadm control --reload-rules
   sudo systemctl daemon-reload
   ```

5. Optional: `rm -rf ~/.config/lilynk` for a clean slate, and
   `sudo gpasswd -d "$USER" lilynk; sudo groupdel lilynk`. If you had no KDE RemoteDesktop
   grant before, the installer seeded `~/.local/share/flatpak/db/kde-authorized`; delete it if
   nothing else relies on it. Your `input` group membership is harmless to keep.
