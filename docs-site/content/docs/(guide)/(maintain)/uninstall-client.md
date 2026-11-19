---
title: Uninstalling a Client
description: Remove the Lilynk app from the device you stream to, and clear what each client leaves behind.
---

Remove the Lilynk app from any device you stream to, and see what each one leaves behind.

## Clients

Removing a client doesn't unpair it: unpair the device in the host's
[web console](/docs/web-console) too.

| Client | Remove | Left behind |
|---|---|---|
| Linux Flatpak | `flatpak uninstall --user --delete-data org.projectcolony.lilynk` | A `~/.config/lilynk` an older version used, if any; the Flatpak remote (`flatpak remotes --user`, then `flatpak remote-delete --user <name>`) |
| Linux packages | `sudo apt purge lilynk-client`, `sudo dnf remove lilynk-client` or `sudo pacman -Rns lilynk-client`; drop the repo [as for hosts](/docs/uninstall#linux-hosts) | `~/.config/lilynk` |
| Windows installer | **Lilynk** in **Settings → Apps → Installed apps**, or `& "$env:LOCALAPPDATA\Programs\Lilynk\unins000.exe" /VERYSILENT`. Portable zip: delete the folder | `%APPDATA%\lilynk` |
| Windows MSIX | `Get-AppxPackage org.projectcolony.lilynk \| Remove-AppxPackage` | `%APPDATA%\lilynk`, if present |
| macOS | Quit it and drag it from **Applications** to the Trash | — |
| iPhone, iPad, Apple TV | Delete the app; if you joined the beta, stop testing in **TestFlight** | — |
| Android, Android TV | Uninstall from Google Play or **Settings → Apps**; leave the testing program on the Play listing if you joined canary | — |
| Steam Deck (Decky) | Uninstall **Lilynk** from Decky's plugin list ([Steam Deck → Uninstalling](/docs/steam-deck#uninstalling)) | The Steam shortcuts, the Steam Input template, the client and its settings: `~/.var/app/org.projectcolony.lilynk` for the Flatpak, `~/.config/lilynk` for a native client |
| LG webOS TV | Remove it from the TV's launcher | — |
