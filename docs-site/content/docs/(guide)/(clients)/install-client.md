---
title: Install a Client
description: Build, install and update the Lilynk app on the device you stream to, on Linux, Steam Deck, Windows, macOS, iPhone, iPad, Apple TV, Vision Pro or Android.
---

Install the Lilynk app on the device you stream *to*, then [pair](/docs/pairing) it with your
host once. To pick between the apps, see [Clients](/docs/clients).

Lilynk publishes no app builds yet: no store listing, package or download. Build the client for
your device from a clone ([Build from source](/docs/developers/build-from-source)). The apps
punktfunk publishes speak punktfunk's protocol and don't connect to a Lilynk host.

## Pick your device

| Device | Install |
|--------|---------|
| Linux desktop or laptop | [Your build, or a package you build](#linux-desktop-flatpak) |
| Steam Deck | [Decky plugin and the Linux client](/docs/steam-deck) |
| Windows 10 or 11 (x64, Arm64) | [Installer](#windows) |
| Mac (macOS 14+) | [Xcode build](#macos) |
| iPhone, iPad, Apple TV (17+), Apple Vision Pro | [Xcode build](#ios-ipados-apple-tv) |
| Android 9+ phone or TV | [APK](#android) |
| Anything else | [Moonlight](/docs/moonlight) |

## Linux desktop [#linux-desktop-flatpak]

Build the client from a clone
([Build from source → Linux client](/docs/developers/build-from-source#linux-client)). For your
package manager, build the `lilynk-client` package instead:

| Distro | Package build |
|--------|---------|
| Ubuntu 26.04 or newer | `packaging/debian/build-client-deb.sh` with `VERSION` set as on [Ubuntu](/docs/ubuntu#2-install-the-host), then `sudo apt install ./dist/lilynk-client_*.deb` |
| Fedora 43 or newer | The RPM build on [Fedora](/docs/fedora#2-install-the-host) also writes `lilynk-client` |
| Arch | The `PKGBUILD` on [Arch Linux](/docs/arch#2-install-the-host) also builds `lilynk-client` |

The package adds **Lilynk** to your app menu, and **Lilynk Console** for the controller interface.
Every client package also installs the [`lilynk` command](/docs/clients#scripting-the-lilynk-cli)
for scripts.

## Windows

1. Build the client and its installer from a clone
   ([Build from source → Windows host and client](/docs/developers/build-from-source#windows-host-and-client)).
2. Run it. No admin prompt: it installs to `%LOCALAPPDATA%\Programs\Lilynk`, registers
   `lilynk://` links, puts `lilynk` on your PATH and fetches the Windows App Runtime if it's
   missing.
3. Open **Lilynk** from the Start menu, or **Lilynk Console** for the controller interface
   (x64 only: the Arm64 build has no console yet).

**Steam overlay and Big Picture.** In Steam, **Add a Non-Steam Game** and browse to
`%LOCALAPPDATA%\Programs\Lilynk\lilynk-client.exe` (or `lilynk-console.exe`).

**Portable zip and MSIX.** The same build also packs as
`lilynk-client-windows_x64-portable.zip` (unzip and run `lilynk-client.exe`; no links, no PATH
entry) and `lilynk-client-windows_x64.msix` (`Add-AppxPackage .\lilynk-client-windows_x64.msix`).
Both need the [Windows App Runtime 2.x](https://learn.microsoft.com/windows/apps/windows-app-sdk/downloads).
The MSIX can't be added to Steam.

## macOS

1. Build the app from a clone with Xcode
   ([Build from source → Apple client](/docs/developers/build-from-source#apple-client)).
2. Drag **Lilynk** to **Applications**.

## iOS, iPadOS, Apple TV

Build the app from a clone with Xcode
([Build from source → Apple client](/docs/developers/build-from-source#apple-client)). One app
covers iPhone, iPad, Apple TV and Apple Vision Pro. A build sideloaded with AltStore, SideStore or
Sideloadly has no Wake-on-LAN and no host list in the widget.

## Android

Build the APK from a clone
([Build from source → Android client](/docs/developers/build-from-source#android-client)) and
install it. The same app runs on phones, tablets and Android TV.

## LG webOS TV [#lg-webos-tv-community]

The webOS app is punktfunk's ([punktfunk/client-webos](https://github.com/punktfunk/client-webos))
and doesn't connect to a Lilynk host. Lilynk has no webOS client.

## Keeping a client up to date

A client and a host don't need the same version. To update the host, see [Updating](/docs/updating).
To update a client, `git pull` in the clone, build it again, and install the new build:

| Client | Install the new build |
|---|---|
| Linux package | Install the new package with your package manager |
| Windows installer | Run the newer installer. Hosts and pairing are kept. |
| Windows MSIX or portable | Add the newer `.msix`, or unzip the newer build over the old one. |
| macOS | Drag the newer app over the old one |
| iPhone, iPad, Apple TV | Install the newer app from Xcode |
| Android | Install the newer APK over the old one |
| Steam Deck (Decky) | [Steam Deck → Updating](/docs/steam-deck#updating) |

**Moving from the MSIX to the installer.** Remove the MSIX first, then run the installer. The MSIX
takes its saved hosts and pairing with it, so pair again once:

```powershell
Get-AppxPackage org.projectcolony.lilynk | Remove-AppxPackage
```

## Removing a client

See [Uninstalling → Clients](/docs/uninstall-client): the command for each client, and what stays
behind on the device and the host.
