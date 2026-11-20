//! `quic` client connector: the connection handle and the wire vocabulary its calls
//! share. Items marked `#[cfg(feature = "quic")]` sit under `LILYNK_FEATURE_QUIC` in the
//! header; the plain `pub const`s here do not.

// Declaration order is header order, as in `lib.rs`; blank lines keep rustfmt from sorting.
mod connect;

mod planes;

mod pad;

mod video;

mod screens;

mod input;

mod host;

mod clipboard;

mod control;

mod au_memory;

mod mgmt;

pub use au_memory::*;
pub use clipboard::*;
pub use connect::*;
pub use control::*;
#[cfg(feature = "quic")]
pub use host::*;
#[cfg(feature = "quic")]
pub use input::*;
#[cfg(feature = "quic")]
pub use mgmt::*;
pub use pad::*;
#[cfg(feature = "quic")]
pub use planes::*;
#[cfg(feature = "quic")]
pub use screens::*;
pub use video::*;

#[cfg(feature = "quic")]
use crate::*;

/// Live `lilynk/2` connection (one QUIC connection and its media, pumped on internal threads).
/// One puller thread per plane; never two threads on the same plane.
#[cfg(feature = "quic")]
pub struct LilynkConnection {
    inner: lilynk_core::client::NativeClient,
    /// Last `next_au` payload. Pointer valid until the next video pull.
    last: std::sync::Mutex<Option<lilynk_core::session::Frame>>,
    /// Last `next_screen_au` payload of screens 1 to 3. Each pointer valid until that screen's
    /// next pull.
    last_screens: std::sync::Mutex<[Option<lilynk_core::session::Frame>; 3]>,
    /// Last `next_audio` payload. Independent of the video slot.
    last_audio: std::sync::Mutex<Option<lilynk_core::client::AudioPacket>>,
    /// In-core PCM decode. Returned pointer valid until the next PCM call.
    audio_pcm: std::sync::Mutex<AudioPcmState>,
    /// Last clipboard payload. Pointer valid until the next `next_clipboard`.
    last_clip: std::sync::Mutex<Option<Vec<u8>>>,
    /// Last cursor RGBA. Pointer valid until the next cursor-shape call.
    last_cursor_shape: std::sync::Mutex<Option<lilynk_core::quic::CursorShape>>,
    /// Stats overlay window as last drained; `hud_text` formats it at any tier.
    hud_snap: std::sync::Mutex<lilynk_core::hud::StatsSnapshot>,
}

// Plane pullers and the demo host's callers use these handles from several threads at once.
#[cfg(feature = "quic")]
const _: fn() = || {
    fn shared<T: Send + Sync>() {}
    shared::<LilynkConnection>();
    shared::<LilynkDemoHost>();
};

/// `LilynkHidOutput::kind`: lightbar RGB (`r`/`g`/`b` valid).
pub const LILYNK_HIDOUT_LED: u8 = 1;
/// `LilynkHidOutput::kind`: player-indicator LEDs (`player_bits` valid, low 5 bits).
pub const LILYNK_HIDOUT_PLAYER_LEDS: u8 = 2;
/// `LilynkHidOutput::kind`: one adaptive-trigger effect (`which` + `effect`/`effect_len` valid).
pub const LILYNK_HIDOUT_TRIGGER: u8 = 3;
/// Trackpad haptic pulse. `which` = side (0 right, 1 left); `effect[0..6]` =
/// amplitude/period/count as LE `u16`, `effect_len = 6`. Drop if no coils.
pub const LILYNK_HIDOUT_TRACKPAD_HAPTIC: u8 = 4;
/// DS5 audio-control region. Samples arrive via [`lilynk_connection_next_pad_audio`].
/// `which` = flags; `effect[0..6]` = report bytes 5..=10; `effect_len = 6`. Change-only.
pub const LILYNK_HIDOUT_AUDIO_CTL: u8 = 5;
/// Raw hidraw report to replay (`HidRaw`). `hid_kind` + `raw`/`raw_len` valid.
/// Only the Steam Controller, Steam Controller 2 and Puck backends emit these.
pub const LILYNK_HIDOUT_HID_RAW: u8 = 6;
/// `LilynkHidOutput::kind`: DualSense microphone-mute LED. `which` = mode (0 off, 1 on,
/// 2 pulse).
pub const LILYNK_HIDOUT_MIC_LED: u8 = 7;
/// Capacity of `LilynkHidOutput::effect` (DualSense trigger parameter block).
pub const LILYNK_HID_EFFECT_MAX: u8 = 11;
// The wire clamps a trigger effect to this length; `effect` must hold it whole.
const _: () = assert!(LILYNK_HID_EFFECT_MAX as usize == lilynk_core::quic::TRIGGER_EFFECT_MAX);

/// HID-output feedback from the host virtual pad ([`lilynk_connection_next_hidout`]).
/// `kind` selects which fields to replay on the physical controller.
#[cfg(feature = "quic")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LilynkHidOutput {
    /// One of `LILYNK_HIDOUT_*`.
    pub kind: u8,
    /// Gamepad index.
    pub pad: u8,
    /// LED: lightbar red.
    pub r: u8,
    /// LED: lightbar green.
    pub g: u8,
    /// LED: lightbar blue.
    pub b: u8,
    /// PlayerLeds: lit player indicators (low 5 bits).
    pub player_bits: u8,
    /// Trigger: 0 = L2, 1 = R2.
    pub which: u8,
    /// Trigger: number of valid bytes in `effect` (≤ `LILYNK_HID_EFFECT_MAX`).
    pub effect_len: u8,
    /// DualSense trigger parameter block. Length is [`LILYNK_HID_EFFECT_MAX`], not a second `11`.
    pub effect: [u8; LILYNK_HID_EFFECT_MAX as usize],
    /// HidRaw channel: `LILYNK_HID_RAW_OUTPUT` or `LILYNK_HID_RAW_FEATURE`.
    pub hid_kind: u8,
    /// HidRaw: number of valid bytes in `raw` (≤ `LILYNK_HID_REPORT_MAX`).
    pub raw_len: u8,
    /// HidRaw report, id byte first. Feature frames may be zero-padded; sized off `HID_REPORT_MAX`.
    pub raw: [u8; lilynk_core::quic::HID_REPORT_MAX],
}

#[cfg(feature = "quic")]
impl LilynkHidOutput {
    /// Map every [`HidOutput`](lilynk_core::quic::HidOutput) variant. `HidRaw` uses `raw`/`hid_kind`.
    fn from_hid(h: &lilynk_core::quic::HidOutput) -> LilynkHidOutput {
        use lilynk_core::quic::HidOutput;
        let mut out = LilynkHidOutput {
            kind: 0,
            pad: 0,
            r: 0,
            g: 0,
            b: 0,
            player_bits: 0,
            which: 0,
            effect_len: 0,
            effect: [0u8; 11],
            hid_kind: 0,
            raw_len: 0,
            raw: [0u8; lilynk_core::quic::HID_REPORT_MAX],
        };
        match h {
            HidOutput::Led { pad, r, g, b } => {
                out.kind = LILYNK_HIDOUT_LED;
                out.pad = *pad;
                out.r = *r;
                out.g = *g;
                out.b = *b;
            }
            HidOutput::PlayerLeds { pad, bits } => {
                out.kind = LILYNK_HIDOUT_PLAYER_LEDS;
                out.pad = *pad;
                out.player_bits = *bits;
            }
            HidOutput::Trigger { pad, which, effect } => {
                out.kind = LILYNK_HIDOUT_TRIGGER;
                out.pad = *pad;
                out.which = *which;
                let n = effect.len().min(out.effect.len());
                out.effect[..n].copy_from_slice(&effect[..n]);
                out.effect_len = n as u8;
            }
            HidOutput::TrackpadHaptic {
                pad,
                side,
                amplitude,
                period,
                count,
            } => {
                // No size guard: pack into `which` + `effect[0..6]` LE, `effect_len = 6`.
                out.kind = LILYNK_HIDOUT_TRACKPAD_HAPTIC;
                out.pad = *pad;
                out.which = *side;
                out.effect[0..2].copy_from_slice(&amplitude.to_le_bytes());
                out.effect[2..4].copy_from_slice(&period.to_le_bytes());
                out.effect[4..6].copy_from_slice(&count.to_le_bytes());
                out.effect_len = 6;
            }
            HidOutput::HidRaw { pad, kind, data } => {
                out.kind = LILYNK_HIDOUT_HID_RAW;
                out.pad = *pad;
                out.hid_kind = *kind;
                // `decode` already bounds; clamp so a local oversize cannot overrun `raw`.
                let n = data.len().min(out.raw.len());
                out.raw[..n].copy_from_slice(&data[..n]);
                out.raw_len = n as u8;
            }
            HidOutput::AudioCtl { pad, flags, raw } => {
                // Same pack as TrackpadHaptic. `pad as u8` is lossless: decode rejects ≥ MAX_PADS.
                out.kind = LILYNK_HIDOUT_AUDIO_CTL;
                out.pad = *pad as u8;
                out.which = *flags;
                out.effect[0..6].copy_from_slice(raw);
                out.effect_len = 6;
            }
            HidOutput::MicLed { pad, mode } => {
                out.kind = LILYNK_HIDOUT_MIC_LED;
                out.pad = *pad;
                out.which = *mode;
            }
        }
        out
    }
}

/// `LilynkRichInput::kind`: a touchpad contact (`finger`/`active`/`x`/`y` valid).
pub const LILYNK_RICH_TOUCHPAD: u8 = 1;
/// `LilynkRichInput::kind`: a motion sample (`gyro`/`accel` valid).
pub const LILYNK_RICH_MOTION: u8 = 2;
/// `RichInput::TouchpadEx` on the wire: surface (0 single / 1 Steam-left / 2
/// Steam-right) plus click + pressure. C send path is size-prefixed
/// `LilynkRichInputEx` via `lilynk_connection_send_rich_input2`.
pub const LILYNK_RICH_TOUCHPAD_EX: u8 = 3;
/// `RichInput::HidReport` on the wire (`[0xCC][0x04][pad][len][data…]`): raw HID
/// input for the host as-is pad (`LILYNK_GAMEPAD_STEAMCONTROLLER2`). C clients
/// send via [`lilynk_connection_send_hid_report`], never by building the datagram.
pub const LILYNK_RICH_HID_REPORT: u8 = 4;

/// [`lilynk_connection_set_sc2_gate`] bit: the client's overlay owns the pad. Its raw state
/// goes out neutral, and a button held now stays off the wire until it is released.
pub const LILYNK_SC2_GATE_MASKED: u32 = 1;
/// [`lilynk_connection_set_sc2_gate`] bit: Steam and QAM stay with the client.
pub const LILYNK_SC2_GATE_SYSTEM_LOCAL: u32 = 2;
/// [`lilynk_connection_set_sc2_gate`] bit: the client opens its ring on Select then A, so
/// that chord stays off the wire.
pub const LILYNK_SC2_GATE_CHORDS: u32 = 4;

/// One rich client→host input for the host virtual DualSense
/// ([`lilynk_connection_send_rich_input`]): touchpad contact or motion sample.
/// Set `kind` and the matching fields; the others are ignored.
#[cfg(feature = "quic")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LilynkRichInput {
    /// One of `LILYNK_RICH_*`.
    pub kind: u8,
    /// Gamepad index.
    pub pad: u8,
    /// Touchpad: contact id (0 or 1).
    pub finger: u8,
    /// Touchpad: 1 = finger down, 0 = lifted.
    pub active: u8,
    /// Touchpad: normalized x, 0..=65535 across the touchpad.
    pub x: u16,
    /// Touchpad: normalized y, 0..=65535 across the touchpad.
    pub y: u16,
    /// Motion: gyro (pitch, yaw, roll), raw signed-16.
    pub gyro: [i16; 3],
    /// Motion: accelerometer (x, y, z), raw signed-16.
    pub accel: [i16; 3],
}

#[cfg(feature = "quic")]
impl LilynkRichInput {
    fn to_rich(self) -> Option<lilynk_core::quic::RichInput> {
        use lilynk_core::quic::RichInput;
        match self.kind {
            LILYNK_RICH_TOUCHPAD => Some(RichInput::Touchpad {
                pad: self.pad,
                finger: self.finger,
                active: self.active != 0,
                x: self.x,
                y: self.y,
            }),
            LILYNK_RICH_MOTION => Some(RichInput::Motion {
                pad: self.pad,
                gyro: self.gyro,
                accel: self.accel,
            }),
            _ => None,
        }
    }
}

/// Superset of [`LilynkRichInput`] for `TouchpadEx` (second pad, click, signed
/// coords, pressure). Set `struct_size = sizeof(LilynkRichInputEx)`.
#[cfg(feature = "quic")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LilynkRichInputEx {
    /// Must equal `sizeof(LilynkRichInputEx)`.
    pub struct_size: u32,
    /// One of `LILYNK_RICH_*` (`TOUCHPAD` / `MOTION` / `TOUCHPAD_EX`).
    pub kind: u8,
    /// Gamepad index.
    pub pad: u8,
    /// Touchpad/TouchpadEx: contact id.
    pub finger: u8,
    /// Touchpad/TouchpadEx: 1 = finger down / touching, 0 = lifted.
    pub active: u8,
    /// TouchpadEx: surface — 0 = single/DualSense, 1 = Steam left pad, 2 = Steam right pad.
    pub surface: u8,
    /// TouchpadEx: 1 = the pad is physically clicked, distinct from a touch contact.
    pub click: u8,
    /// Reserved for alignment; set to 0.
    pub _reserved: [u8; 2],
    /// TouchpadEx: x, **signed**, centred at 0 (Steam report convention). For a
    /// `TOUCHPAD` kind through this struct, store the unsigned `0..=65535` bits.
    pub x: i16,
    /// TouchpadEx: y, signed, centred at 0.
    pub y: i16,
    /// TouchpadEx: contact pressure (`0` if the surface has no force sensor).
    pub pressure: u16,
    /// Motion: gyro (pitch, yaw, roll), raw signed-16.
    pub gyro: [i16; 3],
    /// Motion: accelerometer (x, y, z), raw signed-16.
    pub accel: [i16; 3],
}

#[cfg(feature = "quic")]
impl LilynkRichInputEx {
    fn to_rich(self) -> Option<lilynk_core::quic::RichInput> {
        use lilynk_core::quic::RichInput;
        match self.kind {
            LILYNK_RICH_TOUCHPAD_EX => Some(RichInput::TouchpadEx {
                pad: self.pad,
                surface: self.surface,
                finger: self.finger,
                touch: self.active != 0,
                click: self.click != 0,
                x: self.x,
                y: self.y,
                pressure: self.pressure,
            }),
            LILYNK_RICH_MOTION => Some(RichInput::Motion {
                pad: self.pad,
                gyro: self.gyro,
                accel: self.accel,
            }),
            LILYNK_RICH_TOUCHPAD => Some(RichInput::Touchpad {
                pad: self.pad,
                finger: self.finger,
                active: self.active != 0,
                x: self.x as u16,
                y: self.y as u16,
            }),
            _ => None,
        }
    }
}

/// [`LilynkPenSample::state`] bit: the pen hovers in range (implied by `TOUCHING`).
pub const LILYNK_PEN_IN_RANGE: u8 = 0x01;
/// [`LilynkPenSample::state`] bit: the tip is in contact.
pub const LILYNK_PEN_TOUCHING: u8 = 0x02;
/// [`LilynkPenSample::state`] bit: primary barrel button (or squeeze mapping) held.
pub const LILYNK_PEN_BARREL1: u8 = 0x04;
/// [`LilynkPenSample::state`] bit: secondary barrel button (or double-tap mapping) held.
pub const LILYNK_PEN_BARREL2: u8 = 0x08;
/// [`LilynkPenSample::tool`]: the pen tip.
pub const LILYNK_PEN_TOOL_PEN: u8 = 0;
/// [`LilynkPenSample::tool`]: the eraser. Client-side mode — no hardware eraser
/// end; squeeze/double-tap mapping usually drives this.
pub const LILYNK_PEN_TOOL_ERASER: u8 = 1;
/// Most samples one [`lilynk_connection_send_pen`] call accepts (one wire batch).
pub const LILYNK_PEN_BATCH_MAX: u32 = 8;
/// [`LilynkPenSample::tilt_deg`] sentinel: no tilt reading.
pub const LILYNK_PEN_TILT_UNKNOWN: u8 = 0xFF;
/// Longest profile id, in bytes, that [`LilynkConnectOpts::profile_id`] sends and
/// [`lilynk_connection_profile`] writes (before the NUL).
pub const LILYNK_PROFILE_ID_MAX: usize = 64;
/// [`LilynkPenSample::azimuth_deg`] / `roll_deg` sentinel: no reading.
pub const LILYNK_PEN_ANGLE_UNKNOWN: u16 = 0xFFFF;
/// [`LilynkPenSample::distance`] sentinel: no hover-distance reading.
pub const LILYNK_PEN_DISTANCE_UNKNOWN: u16 = 0xFFFF;

/// Full stylus state at one instant ([`lilynk_connection_send_pen`];
/// `design/pen-tablet-input.md`). Fill every field (`*_UNKNOWN` if missing); the
/// host diffs samples. `x`/`y` are `0.0..=1.0` in video-frame space.
#[cfg(feature = "quic")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LilynkPenSample {
    /// Normalized `0.0..=1.0` across the video frame. Must be finite.
    pub x: f32,
    /// Normalized `0.0..=1.0` across the video frame. Must be finite.
    pub y: f32,
    /// Tip force, `0..=65535` full scale (`0` while hovering).
    pub pressure: u16,
    /// Hover distance `0..=65534` (0 = at the hover floor), or `LILYNK_PEN_DISTANCE_UNKNOWN`.
    pub distance: u16,
    /// Tilt azimuth, degrees `0..=359` clockwise from north, or `LILYNK_PEN_ANGLE_UNKNOWN`.
    pub azimuth_deg: u16,
    /// Barrel roll (Apple Pencil Pro `rollAngle`), degrees `0..=359`, or
    /// `LILYNK_PEN_ANGLE_UNKNOWN`.
    pub roll_deg: u16,
    /// µs since the previous sample in the same call (`0` for the first) — coalesced
    /// capture spacing.
    pub dt_us: u16,
    /// Bitfield of `LILYNK_PEN_*` state bits. Unknown bits are rejected (`InvalidArg`).
    pub state: u8,
    /// `LILYNK_PEN_TOOL_PEN` or `LILYNK_PEN_TOOL_ERASER`.
    pub tool: u8,
    /// Tilt from the surface normal, degrees `0..=90`, or `LILYNK_PEN_TILT_UNKNOWN`.
    pub tilt_deg: u8,
    /// Set to 0.
    pub _reserved: [u8; 3],
}

#[cfg(feature = "quic")]
impl LilynkPenSample {
    /// `None` = invalid field (non-finite coordinate, unknown state bit, unknown tool).
    /// Embedder input is validated strictly, unlike the loss-tolerant wire decode.
    fn to_sample(self) -> Option<lilynk_core::quic::PenSample> {
        use lilynk_core::quic as q;
        let known = q::PEN_IN_RANGE | q::PEN_TOUCHING | q::PEN_BARREL1 | q::PEN_BARREL2;
        if !self.x.is_finite() || !self.y.is_finite() || self.state & !known != 0 {
            return None;
        }
        let tool = match self.tool {
            LILYNK_PEN_TOOL_PEN => q::PenTool::Pen,
            LILYNK_PEN_TOOL_ERASER => q::PenTool::Eraser,
            _ => return None,
        };
        Some(q::PenSample {
            state: self.state,
            tool,
            x: self.x,
            y: self.y,
            pressure: self.pressure,
            distance: self.distance,
            tilt_deg: self.tilt_deg,
            azimuth_deg: self.azimuth_deg,
            roll_deg: self.roll_deg,
            dt_us: self.dt_us,
        })
    }
}

/// [`LilynkTouchpadContact::state`] bit: the finger touches the surface.
pub const LILYNK_TOUCHPAD_TIP: u8 = 0x01;
/// [`LilynkTouchpadContact::state`] bit: a real finger. Clear it for one frame, then lift
/// the contact, to take a gesture back: the host treats the contact as accidental.
pub const LILYNK_TOUCHPAD_CONFIDENT: u8 = 0x02;
/// Most contacts in one [`LilynkTouchpadFrame`].
pub const LILYNK_TOUCHPAD_MAX_CONTACTS: u32 = 5;
/// Highest [`LilynkTouchpadContact::id`].
pub const LILYNK_TOUCHPAD_MAX_ID: u8 = 31;
/// [`lilynk_connection_send_touchpad_pinch`] phase: the pinch starts.
pub const LILYNK_PINCH_BEGIN: u8 = 0;
/// [`lilynk_connection_send_touchpad_pinch`] phase: the pinch moves.
pub const LILYNK_PINCH_UPDATE: u8 = 1;
/// [`lilynk_connection_send_touchpad_pinch`] phase: the pinch ends or is cancelled.
pub const LILYNK_PINCH_END: u8 = 2;

/// One finger on the client's touchpad surface.
#[cfg(feature = "quic")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LilynkTouchpadContact {
    /// Stable for the life of the contact, `0..=LILYNK_TOUCHPAD_MAX_ID`, unique in a frame.
    pub id: u8,
    /// `LILYNK_TOUCHPAD_*` bits. Unknown bits are rejected (`InvalidArg`).
    pub state: u8,
    /// `0..=65535` across the surface, origin top-left.
    pub x: u16,
    /// `0..=65535` down the surface.
    pub y: u16,
}

/// The whole touchpad at one instant ([`lilynk_connection_send_touchpad`]). A contact
/// absent from a frame has lifted.
#[cfg(feature = "quic")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LilynkTouchpadFrame {
    /// The surface in tenths of a millimetre; `0` is unknown and the host assumes 100 × 60 mm.
    pub width_dmm: u16,
    pub height_dmm: u16,
    /// `1` while the pad's button is pressed, else `0`.
    pub button: u8,
    /// Contacts in `contacts`, `0..=LILYNK_TOUCHPAD_MAX_CONTACTS`.
    pub count: u8,
    pub contacts: [LilynkTouchpadContact; 5],
}

#[cfg(feature = "quic")]
impl LilynkTouchpadFrame {
    /// `None` on a count above five or an unknown state bit. Core checks the ids.
    fn to_contacts(self) -> Option<(usize, [lilynk_core::quic::TouchpadContact; 5])> {
        let known = LILYNK_TOUCHPAD_TIP | LILYNK_TOUCHPAD_CONFIDENT;
        let n = usize::from(self.count);
        let raw = self.contacts.get(..n)?;
        let mut out = [lilynk_core::quic::TouchpadContact::default(); 5];
        for (slot, c) in out.iter_mut().zip(raw) {
            if c.state & !known != 0 {
                return None;
            }
            *slot = lilynk_core::quic::TouchpadContact {
                id: c.id,
                tip: c.state & LILYNK_TOUCHPAD_TIP != 0,
                confident: c.state & LILYNK_TOUCHPAD_CONFIDENT != 0,
                x: c.x,
                y: c.y,
            };
        }
        Some((n, out))
    }
}

/// One touch on a touch surface ([`lilynk_connection_touchpad_surface`]).
#[cfg(feature = "quic")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LilynkSurfaceTouch {
    /// The platform's identity for the touch, stable while it is down.
    pub id: u64,
    /// `0.0..=1.0` across the surface, origin top-left. Must be finite; outside values clamp.
    pub x: f32,
    /// `0.0..=1.0` down the surface.
    pub y: f32,
}

/// Compositor preference for [`lilynk_connect_ex`]. `AUTO` lets the host pick.
/// A concrete value is honored only if that backend is available now; else auto-detect.
/// Resolved choice is on `Welcome`.
pub const LILYNK_COMPOSITOR_AUTO: u32 = 0;
/// KWin / KDE Plasma.
pub const LILYNK_COMPOSITOR_KWIN: u32 = 1;
/// wlroots (Sway / River). Older clients sent it for Hyprland too; the host still honors that.
pub const LILYNK_COMPOSITOR_WLROOTS: u32 = 2;
/// Mutter / GNOME.
pub const LILYNK_COMPOSITOR_MUTTER: u32 = 3;
/// gamescope (spawned nested).
pub const LILYNK_COMPOSITOR_GAMESCOPE: u32 = 4;
/// Hyprland.
pub const LILYNK_COMPOSITOR_HYPRLAND: u32 = 5;
/// A Windows host's virtual display. Only ever resolved, never requested.
pub const LILYNK_COMPOSITOR_WINDOWS: u32 = 6;
/// niri (the biri fork, which adds virtual outputs).
pub const LILYNK_COMPOSITOR_NIRI: u32 = 7;
/// Umbriel, Noctalia's wlroots compositor, which creates virtual outputs.
pub const LILYNK_COMPOSITOR_UMBRIEL: u32 = 8;

/// Gamepad-backend preference for [`lilynk_connect_ex2`]: which virtual pad
/// the host creates. Precedence: client choice > `LILYNK_GAMEPAD` env > X-Box 360.
/// `AUTO` (or unrecognized) = host decides. Resolved via [`lilynk_connection_gamepad`].
/// Which hosts take which kind: `pick_gamepad` in lilynk-host.
pub const LILYNK_GAMEPAD_AUTO: u32 = 0;
/// uinput X-Box 360 pad (default — every game speaks XInput).
pub const LILYNK_GAMEPAD_XBOX360: u32 = 1;
/// UHID DualSense (`hid-playstation`): adaptive triggers, lightbar, touchpad, motion.
/// Feedback on [`lilynk_connection_next_hidout`]. Linux UHID / Windows UMDF; else X-Box 360.
pub const LILYNK_GAMEPAD_DUALSENSE: u32 = 2;
/// X-Box One/Series identity on the 360 backend (glyphs). No impulse-trigger rumble:
/// evdev `FF_RUMBLE` has two magnitudes. Windows HID Xbox can; see `next_rumble_cmd2`.
pub const LILYNK_GAMEPAD_XBOXONE: u32 = 3;
/// UHID DualShock 4 (`hid-playstation`): lightbar, touchpad, motion, rumble.
/// Rich-input + HID-output planes, no adaptive triggers / player LEDs / mute.
/// Linux UHID / Windows UMDF; else X-Box 360.
pub const LILYNK_GAMEPAD_DUALSHOCK4: u32 = 4;
/// Classic Steam Controller (`hid-steam` on Linux): one stick + dual trackpads + two
/// grip paddles. Linux and Windows hosts; else Xbox 360.
pub const LILYNK_GAMEPAD_STEAMCONTROLLER: u32 = 5;
/// Steam Deck controller: four back grips, both trackpads, IMU. Steam Input
/// re-grabs with native glyphs when Steam runs on the host. Linux/Windows; else X-Box 360.
pub const LILYNK_GAMEPAD_STEAMDECK: u32 = 6;
/// DualSense Edge: DualSense plus two back buttons + two Fn buttons, so client
/// paddles land on native slots. Linux UHID / Windows UMDF; else X-Box 360.
pub const LILYNK_GAMEPAD_DUALSENSEEDGE: u32 = 7;
/// Nintendo Switch Pro: Nintendo glyphs + positional layout, gyro/accel, HD rumble.
/// Linux UHID `hid-nintendo` / Windows UMDF; else X-Box 360.
pub const LILYNK_GAMEPAD_SWITCHPRO: u32 = 8;
/// Steam Controller 2 as-is passthrough. Linux (usbip, UHID fallback) / Windows; else X-Box 360.
pub const LILYNK_GAMEPAD_STEAMCONTROLLER2: u32 = 9;
/// Steam Controller Puck dongle: native seven-interface topology, four slots.
/// For capture clients that own the physical Puck; wired/BLE SC2 stays `STEAMCONTROLLER2`.
pub const LILYNK_GAMEPAD_STEAMCONTROLLER2_PUCK: u32 = 10;
/// Xbox Elite identity. Linux hosts carry the paddles; Windows hosts drop them.
pub const LILYNK_GAMEPAD_XBOXELITE: u32 = 11;
/// 8BitDo Ultimate 2 Wireless (`2DC8:6012`): L4/R4, back paddles, gyro.
pub const LILYNK_GAMEPAD_8BITDO_ULTIMATE2: u32 = 12;
/// 8BitDo Pro 2 (`2DC8:6003`): back paddles, gyro.
pub const LILYNK_GAMEPAD_8BITDO_PRO2: u32 = 13;
/// 8BitDo Pro 3 (`2DC8:6009`): L4/R4, back paddles, gyro.
pub const LILYNK_GAMEPAD_8BITDO_PRO3: u32 = 14;
/// Wireless HORIPAD for Steam (`0F0D:01AB`): rear buttons, gyro.
pub const LILYNK_GAMEPAD_HORIPAD_STEAM: u32 = 15;
/// Joy-Con pair (`057E:2006` + `2007`): SL/SR as paddles, gyro.
pub const LILYNK_GAMEPAD_JOYCON_PAIR: u32 = 16;
/// Switch 2 Pro Controller (`057E:2069`): GL/GR, gyro. Linux hosts only (usbip).
pub const LILYNK_GAMEPAD_SWITCH2_PRO: u32 = 17;
/// Switch 2 GameCube controller (`057E:2073`): analog triggers. Linux hosts only (usbip).
pub const LILYNK_GAMEPAD_SWITCH2_GAMECUBE: u32 = 18;

/// Extended `InputEvent` gamepad button bits: four back grips (Steam L4/L5/R4/R5 ≙
/// Xbox-Elite P1–P4) + misc/capture, in Moonlight's `buttonFlags2 << 16` namespace.
/// Mirror `input::gamepad::BTN_PADDLE1..4` / `BTN_MISC1`.
pub const LILYNK_GAMEPAD_BTN_PADDLE1: u32 = 0x0001_0000;
pub const LILYNK_GAMEPAD_BTN_PADDLE2: u32 = 0x0002_0000;
pub const LILYNK_GAMEPAD_BTN_PADDLE3: u32 = 0x0004_0000;
pub const LILYNK_GAMEPAD_BTN_PADDLE4: u32 = 0x0008_0000;
pub const LILYNK_GAMEPAD_BTN_MISC1: u32 = 0x0020_0000;

/// Video-capability bit for [`lilynk_connect_ex5`]: the client can decode
/// 10-bit (Main10) HEVC.
pub const LILYNK_VIDEO_CAP_10BIT: u8 = 0x01;
/// Video-capability bit: the client can present BT.2020 PQ HDR10 (implies 10-bit).
pub const LILYNK_VIDEO_CAP_HDR: u8 = 0x02;
/// Video-capability bit: the client can decode full-chroma 4:4:4 HEVC. The host
/// emits 4:4:4 only when this is set, the host opted in, the codec is HEVC or PyroWave, and
/// the GPU supports it — else 4:2:0. Read [`lilynk_connection_chroma_format`].
pub const LILYNK_VIDEO_CAP_444: u8 = 0x04;

/// Codec bit for [`lilynk_connect_ex7`] and [`lilynk_connection_codec`]: H.264 / AVC.
pub const LILYNK_CODEC_H264: u8 = 0x01;
/// Codec bit: H.265 / HEVC — the default codec.
pub const LILYNK_CODEC_HEVC: u8 = 0x02;
/// Codec bit: AV1.
pub const LILYNK_CODEC_AV1: u8 = 0x04;
/// PyroWave. Off the codec ladder: chosen only as `preferred_codec`, or by the host's Encoder
/// pin for a client that names none (`design/pyrowave-codec-plan.md`).
pub const LILYNK_CODEC_PYROWAVE: u8 = 0x08;

/// Host-capability bit: the host applies gamepad-state snapshots (a capable client
/// sends full-state snapshots instead of per-transition events).
pub const LILYNK_HOST_CAP_GAMEPAD_STATE: u8 = 0x01;
/// Host-capability bit: the host supports the shared clipboard; a client may offer the toggle.
pub const LILYNK_HOST_CAP_CLIPBOARD: u8 = 0x02;
/// Host injects stylus. Without it [`lilynk_connection_send_pen`] is `Unsupported`.
/// (`design/pen-tablet-input.md`.)
pub const LILYNK_HOST_CAP_PEN: u8 = 0x10;
/// Host-capability bit: per-gamepad audio (DualSense voice-coil + speaker) on
/// the 0xD1 plane toward pads declared via [`lilynk_connection_set_pad_audio_caps`].
/// Set only when the client asked via [`LILYNK_CLIENT_CAP_PAD_AUDIO`].
pub const LILYNK_HOST_CAP_PAD_AUDIO: u8 = 0x40;
/// Session is on lossless `0xD3`, not Opus. Distinguishes 48 kHz/16-bit PCM from
/// 48 kHz Opus when draining [`lilynk_connection_next_audio`]. PCM decode path
/// does not need it; still read [`lilynk_connection_audio_sample_rate`].
pub const LILYNK_HOST_CAP_AUDIO_HIRES: u8 = 0x80;

/// Host-capability bit in [`lilynk_connection_host_caps2`] (second byte): the
/// host injector puts wire touch contacts on its desktop. Without the bit, fall
/// back to a cursor model — the host drops every contact silently.
pub const LILYNK_HOST_CAP2_TOUCH: u8 = 0x02;

/// Pad-audio `kind` ([`lilynk_connection_next_pad_audio`]): BACK channel pair —
/// DualSense voice-coil haptics, 5 ms Opus frames.
pub const LILYNK_PAD_AUDIO_KIND_HAPTICS: u8 = 0;
/// Pad-audio `kind`: FRONT channel pair — the controller's built-in speaker, 10 ms Opus frames.
pub const LILYNK_PAD_AUDIO_KIND_SPEAKER: u8 = 1;

/// [`lilynk_connection_set_pad_audio_caps`] bit: the pad renders the HAPTICS stream
/// (DualSense voice coils).
pub const LILYNK_PAD_AUDIO_CAP_HAPTICS: u8 = 0x01;
/// [`lilynk_connection_set_pad_audio_caps`] bit: the pad renders the SPEAKER stream.
pub const LILYNK_PAD_AUDIO_CAP_SPEAKER: u8 = 0x02;

// ABI cap bits must match the wire constants.
#[cfg(feature = "quic")]
const _: () = {
    assert!(LILYNK_VIDEO_CAP_10BIT == lilynk_core::quic::VIDEO_CAP_10BIT);
    assert!(LILYNK_VIDEO_CAP_HDR == lilynk_core::quic::VIDEO_CAP_HDR);
    assert!(LILYNK_VIDEO_CAP_444 == lilynk_core::quic::VIDEO_CAP_444);
    assert!(LILYNK_CODEC_H264 == lilynk_core::quic::CODEC_H264);
    assert!(LILYNK_CODEC_HEVC == lilynk_core::quic::CODEC_HEVC);
    assert!(LILYNK_CODEC_AV1 == lilynk_core::quic::CODEC_AV1);
    assert!(LILYNK_CODEC_PYROWAVE == lilynk_core::quic::CODEC_PYROWAVE);
    assert!(LILYNK_HOST_CAP_GAMEPAD_STATE == lilynk_core::quic::HOST_CAP_GAMEPAD_STATE);
    assert!(LILYNK_HOST_CAP_CLIPBOARD == lilynk_core::quic::HOST_CAP_CLIPBOARD);
    assert!(LILYNK_HOST_CAP_PEN == lilynk_core::quic::HOST_CAP_PEN);
    assert!(LILYNK_HOST_CAP_PAD_AUDIO == lilynk_core::quic::HOST_CAP_PAD_AUDIO);
    assert!(LILYNK_HOST_CAP_AUDIO_HIRES == lilynk_core::quic::HOST_CAP_AUDIO_HIRES);
    assert!(LILYNK_HOST_CAP2_TOUCH == lilynk_core::quic::HOST_CAP2_TOUCH);
    assert!(LILYNK_CLIENT_CAP_PAD_AUDIO == lilynk_core::quic::CLIENT_CAP_PAD_AUDIO);
    assert!(LILYNK_CLIENT_CAP_AUDIO_HIRES == lilynk_core::quic::CLIENT_CAP_AUDIO_HIRES);
    assert!(LILYNK_CLIENT_CAP_KEEP_HOST_AUDIO == lilynk_core::quic::CLIENT_CAP_KEEP_HOST_AUDIO);
    assert!(LILYNK_PAD_AUDIO_KIND_HAPTICS == lilynk_core::quic::PAD_AUDIO_KIND_HAPTICS);
    assert!(LILYNK_PAD_AUDIO_KIND_SPEAKER == lilynk_core::quic::PAD_AUDIO_KIND_SPEAKER);
    // Setter cap bits are arrival flags 8/9 shifted down.
    assert!(
        (LILYNK_PAD_AUDIO_CAP_HAPTICS as u32) << 8
            == lilynk_core::input::ARRIVAL_FLAG_PAD_AUDIO_HAPTICS
    );
    assert!(
        (LILYNK_PAD_AUDIO_CAP_SPEAKER as u32) << 8
            == lilynk_core::input::ARRIVAL_FLAG_PAD_AUDIO_SPEAKER
    );
    assert!(LILYNK_PEN_IN_RANGE == lilynk_core::quic::PEN_IN_RANGE);
    assert!(LILYNK_PEN_TOUCHING == lilynk_core::quic::PEN_TOUCHING);
    assert!(LILYNK_PEN_BARREL1 == lilynk_core::quic::PEN_BARREL1);
    assert!(LILYNK_PEN_BARREL2 == lilynk_core::quic::PEN_BARREL2);
    assert!(LILYNK_PEN_BATCH_MAX as usize == lilynk_core::quic::PEN_BATCH_MAX);
    assert!(LILYNK_PEN_TILT_UNKNOWN == lilynk_core::quic::PEN_TILT_UNKNOWN);
    assert!(LILYNK_PEN_ANGLE_UNKNOWN == lilynk_core::quic::PEN_ANGLE_UNKNOWN);
    assert!(LILYNK_PEN_DISTANCE_UNKNOWN == lilynk_core::quic::PEN_DISTANCE_UNKNOWN);
    assert!(LILYNK_SC2_GATE_MASKED == lilynk_core::client::SC2_GATE_MASKED);
    assert!(LILYNK_SC2_GATE_SYSTEM_LOCAL == lilynk_core::client::SC2_GATE_SYSTEM_LOCAL);
    assert!(LILYNK_SC2_GATE_CHORDS == lilynk_core::client::SC2_GATE_CHORDS);
};

// ABI gamepad constants must match the wire enum.
const _: () = {
    use lilynk_core::config::GamepadPref;
    use lilynk_core::input::gamepad as g;
    assert!(LILYNK_GAMEPAD_AUTO == GamepadPref::Auto.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_XBOX360 == GamepadPref::Xbox360.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_DUALSENSE == GamepadPref::DualSense.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_XBOXONE == GamepadPref::XboxOne.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_DUALSHOCK4 == GamepadPref::DualShock4.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_STEAMCONTROLLER == GamepadPref::SteamController.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_STEAMDECK == GamepadPref::SteamDeck.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_DUALSENSEEDGE == GamepadPref::DualSenseEdge.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_SWITCHPRO == GamepadPref::SwitchPro.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_STEAMCONTROLLER2 == GamepadPref::SteamController2.to_u8() as u32);
    assert!(
        LILYNK_GAMEPAD_STEAMCONTROLLER2_PUCK == GamepadPref::SteamController2Puck.to_u8() as u32
    );
    assert!(LILYNK_GAMEPAD_XBOXELITE == GamepadPref::XboxElite.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_8BITDO_ULTIMATE2 == GamepadPref::EightBitDoUltimate2.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_8BITDO_PRO2 == GamepadPref::EightBitDoPro2.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_8BITDO_PRO3 == GamepadPref::EightBitDoPro3.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_HORIPAD_STEAM == GamepadPref::HoripadSteam.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_JOYCON_PAIR == GamepadPref::JoyConPair.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_SWITCH2_PRO == GamepadPref::Switch2Pro.to_u8() as u32);
    assert!(LILYNK_GAMEPAD_SWITCH2_GAMECUBE == GamepadPref::Switch2GameCube.to_u8() as u32);
    // Extended button bits mirror the wire `input::gamepad` constants.
    assert!(LILYNK_GAMEPAD_BTN_PADDLE1 == g::BTN_PADDLE1);
    assert!(LILYNK_GAMEPAD_BTN_PADDLE2 == g::BTN_PADDLE2);
    assert!(LILYNK_GAMEPAD_BTN_PADDLE3 == g::BTN_PADDLE3);
    assert!(LILYNK_GAMEPAD_BTN_PADDLE4 == g::BTN_PADDLE4);
    assert!(LILYNK_GAMEPAD_BTN_MISC1 == g::BTN_MISC1);
};

// No `struct_size`: growing these corrupts old callers. Additive kinds must not
// grow them; a deliberate widen needs an [`lilynk_core::ABI_VERSION`] bump. RichInput
// is frozen at 20. HidOutput is 19 + 2 + `HID_REPORT_MAX`.
#[cfg(feature = "quic")]
const _: () = {
    assert!(core::mem::size_of::<LilynkRichInput>() == 20);
    assert!(core::mem::size_of::<LilynkHidOutput>() == 19 + 2 + lilynk_core::quic::HID_REPORT_MAX);
};

#[cfg(all(test, feature = "quic"))]
mod tests {
    use super::*;

    /// AudioCtl packs as kind 5, `which` = flags, `effect[0..6]`, `effect_len = 6`.
    #[test]
    fn hidout_abi_maps_audio_ctl() {
        let out = LilynkHidOutput::from_hid(&lilynk_core::quic::HidOutput::AudioCtl {
            pad: 3,
            flags: 0x17,
            raw: [0x50, 0x60, 0x70, 0x05, 0, 0],
        });
        assert_eq!(out.kind, LILYNK_HIDOUT_AUDIO_CTL);
        assert_eq!(out.pad, 3);
        assert_eq!(out.which, 0x17);
        assert_eq!(out.effect_len, 6);
        assert_eq!(out.effect[..6], [0x50, 0x60, 0x70, 0x05, 0, 0]);
        assert_eq!(out.effect[6..], [0; 5]);
        assert_eq!(out.raw_len, 0);
    }

    /// MicLed maps to kind 7 with the mode in `which`.
    #[test]
    fn hidout_abi_maps_mic_led() {
        let out =
            LilynkHidOutput::from_hid(&lilynk_core::quic::HidOutput::MicLed { pad: 2, mode: 2 });
        assert_eq!(out.kind, LILYNK_HIDOUT_MIC_LED);
        assert_eq!(out.pad, 2);
        assert_eq!(out.which, 2);
        assert_eq!(out.effect_len, 0);
    }

    /// HidRaw maps to kind 6 + `hid_kind`/`raw`/`raw_len`, not a skip.
    #[test]
    fn hidout_abi_maps_hid_raw() {
        // OUTPUT report (id 0x80), host-trimmed to its declared 10 bytes.
        let rumble: Vec<u8> = vec![0x80, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let out = LilynkHidOutput::from_hid(&lilynk_core::quic::HidOutput::HidRaw {
            pad: 2,
            kind: lilynk_core::quic::HID_RAW_OUTPUT,
            data: rumble.clone(),
        });
        assert_eq!(out.kind, LILYNK_HIDOUT_HID_RAW);
        assert_eq!(out.pad, 2);
        assert_eq!(out.hid_kind, lilynk_core::quic::HID_RAW_OUTPUT);
        assert_eq!(out.raw_len, 10);
        assert_eq!(out.raw[..10], rumble[..]);
        assert_eq!(out.raw[10..], [0; lilynk_core::quic::HID_REPORT_MAX - 10]);
        // The other fields stay zero — `kind` alone says which ones are meaningful.
        assert_eq!(out.effect_len, 0);

        // A FEATURE frame arrives whole (zero-padded) and must round-trip whole;
        // anything longer clamps instead of overrunning.
        let mut lizard = vec![0u8; lilynk_core::quic::HID_REPORT_MAX + 8];
        lizard[..6].copy_from_slice(&[0x01, 0x87, 0x03, 0x09, 0x00, 0x00]);
        let out = LilynkHidOutput::from_hid(&lilynk_core::quic::HidOutput::HidRaw {
            pad: 0,
            kind: lilynk_core::quic::HID_RAW_FEATURE,
            data: lizard.clone(),
        });
        assert_eq!(out.hid_kind, lilynk_core::quic::HID_RAW_FEATURE);
        assert_eq!(out.raw_len as usize, lilynk_core::quic::HID_REPORT_MAX);
        assert_eq!(out.raw[..], lizard[..lilynk_core::quic::HID_REPORT_MAX]);
    }
}
