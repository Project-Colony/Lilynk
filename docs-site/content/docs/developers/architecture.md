---
title: Architecture
description: The crates and clients, how a frame and an input event travel through them, and the rules every change keeps.
---

A map of the code: the parts, the path of a frame and an input event, and the invariants a change
must keep. The user-level version is [How it works](/docs/how-it-works).

## A frame, host to glass

1. **Mode.** The client's Hello names `WxH@Hz`. The `ly-vdisplay` crate creates a virtual output at
   that mode through the compositor's backend (KWin, Mutter, Hyprland, niri, Umbriel, wlroots, gamescope) or, on
   Windows, through the IddCx driver of the same name.
2. **Capture.** On Linux, `ly-capture` takes dmabufs straight from the compositor over
   `ext-image-copy-capture-v1` where it can, and otherwise over PipeWire (the ScreenCast portal, or
   KWin's or niri's own screencast). `ly-zerocopy` imports them into CUDA or Vulkan without a CPU copy.
3. **Encode.** On Linux, `ly-encode` opens one backend: NVENC, Vulkan Video, VAAPI, PyroWave or
   software openh264 (`LILYNK_ENCODER` pins one). On Windows the driver encodes inside
   WUDFHost with `ly-encode-win` (NVENC, AMF, QSV, Media Foundation, PyroWave) and hands access
   units to the host through a shared section defined in `ly-driver-proto`.
4. **Send.** `lilynk-core` splits each access unit into packets, adds FEC, seals them with
   AES-GCM (ChaCha20-Poly1305 for clients without hardware AES), and the host paces them out over
   UDP on native threads.
5. **Decode.** The client's session pump (`ly-client-core`) reassembles and decrypts, then walks the
   decode ladder (`ly-client-video`): `ly-vkdecode` (Vulkan Video), `ly-vaapi` or `ly-v4l2dec`
   (Linux) or `ly-dxvadec` (Windows) over `ly-bitstream`'s access-unit plans, then CPU openh264 or
   rav1d. PyroWave decodes in Vulkan compute.
6. **Present.** `ly-presenter` draws the picture into an SDL3 window through an ash swapchain, and
   `ly-console-ui` composites the Skia overlay on top. Apple decodes with VideoToolbox and draws
   with Metal; Android uses `AMediaCodec`. Both link `lilynk-core`.

Audio rides the same session: desktop audio goes to the client as Opus, the client's microphone
comes back.

## An input event, glass to host

1. The client captures keyboard, mouse, touch, pen and gamepads (SDL3 in `ly-presenter` on the
   desktop clients).
2. `lilynk-core` encodes them as input events and sends them as QUIC datagrams.
3. The host hands them to `ly-inject`. On Linux that is libei, KWin fake input or the wlroots
   virtual pointer and keyboard, plus virtual pads over uhid or USB/IP. On Windows mouse and keyboard
   leave as reports from the UMDF HID mouse driver, with `SendInput` for text and as the fallback,
   plus the UMDF gamepad drivers.
4. Rumble, lights and other HID output go back to the client on the same session.

## Two protocols

Both run in one `lilynk-host serve` process.

| | `punktfunk/2` | GameStream |
|---|---|---|
| Clients | The native clients | Moonlight |
| On | Always | `serve --gamestream` (trusted LAN only) |
| Control | QUIC: SPAKE2 pairing, mode changes, clock sync, adaptive bitrate, clipboard | HTTPS pairing (nvhttp), RTSP, ENet |
| Data | The QUIC port's UDP path, GF(2¹⁶) Leopard FEC, keys from the TLS exporter | UDP, GF(2⁸) Reed–Solomon FEC |
| Discovery | mDNS `_punktfunk._udp` | mDNS `_nvstream._tcp` |
| Code | `lilynk-core`, `lilynk-host/src/native*` | `lilynk-host/src/gamestream` |

`--no-default-features --features pyrowave` builds a host without the GameStream code. A third
plane, WebTransport for the browser client, is off unless `--webtransport` is passed. Port numbers:
[Ports](/docs/ports).

`punktfunk/2` is ALPN `pkf2`, and timestamps run on a per-session clock. The browser plane serves
it at `/pf2`. A client that offers only `pkf1` is closed with the wire-version code, which it shows
as "update both".

## Control plane and management API

The management API (`crates/host/lilynk-host/src/mgmt`) is axum over HTTPS on the host's identity
certificate, versioned under `/api/v1`. It takes three credentials:

- a paired device, by its client certificate or a browser's device token, from anywhere: status
  reads, log upload, host actions and ending its own games;
- a plugin's scoped bearer token, from loopback only: an allowlist of admin routes;
- the admin bearer token in the host's `mgmt-token` file, from loopback only: everything.

utoipa generates the OpenAPI spec (`lilynk-host openapi` → `api/openapi.json`). The web console,
the SDK, `lilynk-host ctl` and the plugin runner all talk to it. Endpoints and auth in detail:
[Management API](/docs/developers/management-api).

## Web console

`web/` is a TanStack Start app, server-rendered on Bun (Nitro `bun` preset). It serves HTTPS with
the host's own certificate and gates every request behind a password and a sealed session cookie.
Its server holds the management token and proxies `/api/**` to the loopback API, so the token never
reaches the browser. The API client is generated by orval from `api/openapi.json`; strings live in
`l10n/messages/web/`, with the shared ones in `l10n/messages/common/` and `host/`. Setup: [Build from source](/docs/developers/build-from-source#web-console-runner-and-docs-site).

## Scripting runner and plugins

`sdk/` is `@lilynk/host`, the TypeScript client for the management API, plus the runner
`lilynk-scripting` (`sdk/src/runner-cli.ts`). The runner is a Bun process: a systemd user unit on
Linux, the `LilynkScripting` scheduled task as `LocalService` on Windows. It runs loose scripts
from `~/.config/lilynk/scripts/` and `lilynk-plugin-*` packages from
`~/.config/lilynk/plugins/`. On Linux each plugin gets its own bwrap sandbox
(`sdk/src/sandbox.ts`) and a scoped plugin token. `plugin-kit/` (`@lilynk/plugin-kit`) is the
authoring kit; plugins add game-library sources and automation. Writing one:
[Writing plugins](/docs/developers/writing-plugins).

## Windows drivers

`packaging/windows/drivers/` is its own cargo workspace, built with the WDK:

| Crate | Role |
|---|---|
| `ly-vdisplay` | IddCx virtual display. Encodes in WUDFHost and publishes access units to the host. |
| `ly-gamepad` | HID minidriver that presents the pad the host names: PlayStation, Xbox, Steam, Switch, 8BitDo and Hori models. |
| `ly-xusb` | Xbox 360 pad visible to XInput, with no kernel bus driver. |
| `ly-mouse` | Resident virtual HID mouse and keyboard: Windows draws a cursor on a headless host, and input reaches raw input as a device's. |
| `ly-touchpad` | Per-session virtual Precision Touchpad: a client's touchpad contacts reach Windows as a touchpad. |
| `wdk-iddcx`, `ly-umdf-util` | IddCx bindings and the safe UMDF layer the drivers share. |

`crates/host/ly-driver-proto` is the host↔driver contract, a path dependency of both workspaces. The
host refuses a driver older than `MIN_DRIVER_PROTOCOL_VERSION`, so the installer ships host and
driver together. `packaging/windows/ly-vkhdr-layer` is a Vulkan layer that lets Vulkan games turn on
HDR over the virtual display.

## Where the code lives

Rust crates sit in `crates/<group>/`. `core` and `codec` serve both sides. `client` and `host`
never depend on each other; `scripts/ci/check-crate-groups.py` fails CI if one does. `install`
holds the installers, the tray and the update helper.

| Path | What |
|---|---|
| `crates/core/lilynk-core` | Wire format, FEC, crypto, pacing and the QUIC control plane (`quic` feature) |
| `crates/core/ly-dmabuf` | dmabuf fence wait and read-only mapping, shared by host and clients (Linux) |
| `crates/core/ly-update-check` | Signed update manifest, shared by the host, the update helper and the clients |
| `crates/core/ly-fluent` | WinUI theme colours and motion for the Windows client and installer |
| `crates/codec/ly-bitstream` | Access-unit parsing and decode plans for H.264, HEVC and AV1 |
| `crates/codec/ly-vkdecode`, `ly-vaapi`, `ly-dxvadec` | Native decode rungs: Vulkan Video, VAAPI buffers, D3D11VA |
| `crates/codec/ly-libva` | The dlopen'd libva the VAAPI decoder and encoder share |
| `crates/codec/ly-v4l2`, `ly-v4l2dec` | V4L2 decode: the ioctl half; the uAPI and decoder flows, CPU-testable |
| `crates/codec/pyrowave-sys`, `libvpl-sys` | Vendored PyroWave and Intel VPL, built from source |
| `crates/client/lilynk-ffi` | The C ABI over the core, and the demo-mode loopback host |
| `crates/client/ly-client-core` | Client plumbing: session pump, audio, gamepads, trust, discovery |
| `crates/client/ly-client-video` | Decode ladder, Vulkan device handoff, colour maths and PyroWave decode |
| `crates/client/ly-presenter` | SDL3 window, Vulkan swapchain, input capture |
| `crates/client/ly-console-ui` | Skia console: stats overlay, gamepad home, library, settings |
| `crates/host/lilynk-host` | The host: sessions, both protocol planes, management API, game library, CLI |
| `crates/host/lilynk-encode-worker` | Separate binary that runs PyroWave and the NVIDIA frame convert at raised GPU priority |
| `crates/host/lilynk-capture-worker` | Windows Graphics Capture and the encoder for a mirror or shared-screen session, as the signed-in user |
| `crates/host/ly-vdisplay` | Virtual outputs, one backend per compositor plus the Windows driver backend |
| `crates/host/ly-capture` | Capture behind `Capturer`: PipeWire and direct `ext-image-copy-capture-v1` (Linux); driver push, WGC and remote encoders (Windows) |
| `crates/host/ly-encode-core` | The `Encoder` trait and the policy every backend shares |
| `crates/host/ly-encode`, `ly-encode-win` | Encoder backends on Linux and Windows |
| `crates/host/ly-encode-session` | Windows pool slot to encoder to access-unit section, shared by the display driver and the capture worker |
| `crates/host/ly-zerocopy` | CUDA, EGL and Vulkan dmabuf import (Linux) |
| `crates/host/ly-inject` | Keyboard, mouse, pen and virtual-gamepad injection |
| `crates/host/ly-portal` | The one tokio runtime every portal call runs on (Linux) |
| `crates/host/ly-clipboard` | Shared clipboard backends and the clipboard plane |
| `crates/host/ly-audio` | Desktop audio capture, the virtual microphone, pad audio and the Windows wiring plan |
| `crates/host/ly-frame`, `ly-gpu`, `ly-win-display` | Frame vocabulary; GPU selection; Windows display topology |
| `crates/host/ly-host-config` | The settings registry, its store and the env knobs |
| `crates/host/ly-paths`, `ly-paths-win` | Config directory and owner-private files; Windows DACL checks |
| `crates/host/ly-driver-proto` | Host↔driver contract |
| `crates/host/ly-seats`, `lilynk-seats` | Seat supervisor: inside the service on Windows; the `lilynk-seats` root daemon and its pad broker on Linux |
| `crates/host/ly-seat-keeper`, `ly-seat-audio` | A Windows seat's RDP keeper, its own workspace for IronRDP's lockfile; the seat's audio endpoint DLL |
| `crates/install/lilynk-setup`, `lilynk-setup-win` | Guided Linux installer (`install.sh` fetches it); Windows installer wizard |
| `crates/install/lilynk-tray` | Tray status icon |
| `crates/install/ly-update` | Root helper for console-started updates |
| `clients/session` | `lilynk-session`, the Vulkan stream every desktop front-end spawns |
| `clients/linux`, `clients/windows` | GTK4 and WinUI 3 shells (`lilynk-client`) |
| `clients/cli` | `lilynk`, the headless CLI |
| `clients/probe` | `lilynk-probe`, the reference client and measurement tool |
| `clients/apple` | Swift app for macOS, iOS, iPadOS, tvOS and visionOS; `native/` is its static library |
| `clients/android` | Kotlin app for phone and TV; `native/` is its JNI core |
| `clients/decky` | Steam Deck plugin that launches the CLI |
| `clients/shared` | Test vectors every client checks against |
| `web/`, `sdk/`, `plugin-kit/` | Web console; SDK and runner; plugin authoring kit |
| `l10n/` | Every message a person reads, and `ly-l10n`, which compiles and checks them |
| `api/openapi.json`, `include/lilynk_core.h` | Generated API spec and C header, checked in |
| `packaging/` | apt, rpm, Arch, Flatpak, Bazzite, bootc, Nix, Windows installer and drivers, winget |
| `docs-site/` | This site |
| `tools/` | `gen-headers`; probes and measurement tools such as `loss-harness`, `scroll-probe`, `cursor-probe`, `wl-scanout-probe`, `wgc-probe` and `display-disturb`; the `latency-probe` glass timer page |
| `ci/`, `scripts/` | CI builder images; install, packaging and CI scripts |

The browser, LG webOS, Nintendo Switch and PlayStation 5 clients live in their own repositories and
pin these crates as git dependencies.

## Design invariants

- **One core, linked everywhere.** Protocol, FEC and crypto live once in `lilynk-core`, behind a
  versioned C ABI (`lilynk-ffi`). `lilynk_abi_version()` and `LilynkConfig.struct_size`
  let an embedder detect a mismatch instead of corrupting a struct.
- **No async on the hot path.** The per-frame pipeline runs on native threads. `tokio` and `quinn`
  sit behind the off-by-default `quic` feature and serve the control plane only.
- **The virtual display runs at the client's mode.** Each session gets an output at the client's
  `WxH@Hz`; every compositor keeps its own backend behind the `VirtualDisplay` trait. When the
  client names a smaller view (render scale above 1×), the host scales the picture down before it
  encodes; any other scaling happens in the client's presenter.
- **FEC sized to the protocol.** GF(2⁸) (at most 255 shards a block) for Moonlight; GF(2¹⁶) (up
  to 65535 shards, O(n log n)) for `punktfunk/2`.
- **Each version moves alone.** The ALPN (`pkf2`) versions the wire. `ABI_VERSION` tracks the C
  surface; `PROTOCOL_VERSION` in `ly-driver-proto` tracks the driver. Never ride one bump on
  another.
