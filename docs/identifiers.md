# Identifiers

A reverse-DNS identifier is `org.projectcolony.lilynk`, all lowercase. A second part is a lowercase
dotted suffix: `org.projectcolony.lilynk.host`. Linux and Windows compare these case-sensitively, so a
match that ignores case is no match. A name that follows a platform's own convention keeps that
convention instead.

## Reverse-DNS identifiers

| Platform | Identifiers | Defined in |
| --- | --- | --- |
| Apple | Bundle `org.projectcolony.lilynk`, widgets `org.projectcolony.lilynk.widgets`, App Group `group.org.projectcolony.lilynk` | `clients/apple/Lilynk.xcodeproj` |
| Android | Application id and package `org.projectcolony.lilynk` | `clients/android/app/build.gradle.kts` |
| Linux client | Flatpak, AppStream, GTK and Wayland app id, icon `org.projectcolony.lilynk`; console window app id `org.projectcolony.lilynk.console`; launchers `org.projectcolony.lilynk.desktop` and `org.projectcolony.lilynk.console.desktop` | `packaging/flatpak/`, `packaging/linux/`, `crates/client/ly-presenter/` |
| Linux host | KWin grant `org.projectcolony.lilynk.host.desktop`, launchers `org.projectcolony.lilynk.starthost.desktop` and `org.projectcolony.lilynk.webconsole.desktop`, tray autostart `org.projectcolony.lilynk.tray.desktop`, D-Bus `org.projectcolony.lilynk.shell`, polkit `org.projectcolony.lilynk.dm-helper` | `packaging/linux/`, `crates/host/` |
| Windows | MSIX `org.projectcolony.lilynk`, AUMIDs `org.projectcolony.lilynk.client` and `org.projectcolony.lilynk.tray` | `clients/windows/`, `crates/install/` |

## Platform conventions

| Kind | Form | Example |
| --- | --- | --- |
| Packages, binaries, systemd units, sysext images | Lowercase, hyphenated | `lilynk-host`, `lilynk-client.raw` |
| Windows services, tasks, registry keys | CamelCase, no dots | `LilynkHost`, `HKLM\SOFTWARE\Punktfunk` |
| winget | Publisher.Package | `Lilynk.LilynkHost` |
| URL scheme, DNS-SD service | Lowercase | `punktfunk://`, `_punktfunk._udp` |
| GNOME Shell extension | name@domain | `lilynk-windows@projectcolony.org` |

## Never rename

- **Apple bundle id and App Group.** A new bundle id is a new App Store app, and the keychain
  identity stays with the old one.
- **Android application id.** Play ties the listing to it.
- **Windows Add/Remove Programs GUIDs and the winget `ProductCode`.** Upgrades find the install by
  them.
- **Plugin AppContainers, `lilynk.plugin.<id>`.** The name derives the SID that every folder
  grant names.
- **The host sysext image `punktfunk`.** systemd-sysext merges an image only when its
  `extension-release` name matches the file, and the feed's updater writes that file.

## Older names still read

Older releases shipped `org.projectcolony.Lilynk` on Linux and `lilynk.*` on Windows. The code
that moves those installs names the old form on purpose:

- The Flatpak remote keeps the old refs, end-of-life with a rebase onto the new id
  (`.gitea/workflows/flatpak.yml`).
- Decky, its launch wrapper and the installer drive an old Flatpak until it moves.
- The host renames the tray's old autostart entry, and the Windows installer deletes the old tray
  AUMID key.
- Native packages link the old icon name to the new one for desktop entries users already have.
