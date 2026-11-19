//! The raw media session: packetize and FEC on one side, reassemble on the other, over an
//! in-process loopback. The C harness drives it; a client uses the connection API instead.

use crate::*;
use lilynk_core::config::{Config, FecConfig, FecScheme, Role};
#[cfg(feature = "quic")]
use lilynk_core::input::InputEvent;
use lilynk_core::session::{MediaV2, Session};
use lilynk_core::stats::Stats;
use lilynk_core::transport::loopback_pair;

/// Opaque session handle. C sees only the pointer.
pub struct LilynkSession {
    inner: Session,
    /// Last polled frame. [`LilynkFrame::data`] is valid until the next poll/free.
    last_frame: Option<lilynk_core::session::Frame>,
}

/// Session configuration. Set `struct_size` to `sizeof(LilynkConfig)`; a
/// smaller prefix is rejected rather than over-read.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LilynkConfig {
    pub struct_size: u32,
    /// 0 = host, 1 = client.
    pub role: u32,
    /// 0 = GF(2⁸), 1 = GF(2¹⁶).
    pub fec_scheme: u32,
    pub fec_percent: u32,
    pub max_data_per_block: u32,
    pub shard_payload: u32,
    /// Test hook for the loopback transport; 0 in production.
    pub loopback_drop_period: u32,
    /// Largest encoded access unit the receiver accepts (reassembler memory bound).
    pub max_frame_bytes: u64,
}

impl LilynkConfig {
    fn to_config(self) -> Result<Config, LilynkStatus> {
        let role = match self.role {
            0 => Role::Host,
            1 => Role::Client,
            _ => return Err(LilynkStatus::InvalidArg),
        };
        // Reject before narrowing: 300% or a 65600-shard block must not wrap to a valid u8/u16.
        let scheme = u8::try_from(self.fec_scheme)
            .ok()
            .and_then(FecScheme::from_u8)
            .ok_or(LilynkStatus::InvalidArg)?;
        let fec_percent = u8::try_from(self.fec_percent).map_err(|_| LilynkStatus::InvalidArg)?;
        let max_data_per_block =
            u16::try_from(self.max_data_per_block).map_err(|_| LilynkStatus::InvalidArg)?;
        // 32-bit: `as usize` truncates >4 GiB to a residue that still passes `validate()`.
        let max_frame_bytes =
            usize::try_from(self.max_frame_bytes).map_err(|_| LilynkStatus::InvalidArg)?;
        let cfg = Config {
            role,
            fec: FecConfig {
                scheme,
                fec_percent,
                max_data_per_block,
            },
            shard_payload: self.shard_payload as usize,
            max_frame_bytes,
            loopback_drop_period: self.loopback_drop_period,
        };
        cfg.validate().map_err(|e| e.status())?;
        Ok(cfg)
    }
}

/// Read `struct_size` first so a smaller older layout is rejected, not over-read.
///
/// # Safety
/// `cfg` is null or points to at least its declared `struct_size` bytes.
unsafe fn config_from_ptr(cfg: *const LilynkConfig) -> Result<Config, LilynkStatus> {
    if cfg.is_null() {
        return Err(LilynkStatus::NullPointer);
    }
    // SAFETY: `addr_of!` does not form a `&`; the caller may have a smaller older layout.
    let declared = unsafe { std::ptr::addr_of!((*cfg).struct_size).read_unaligned() } as usize;
    if declared < std::mem::size_of::<LilynkConfig>() {
        return Err(LilynkStatus::InvalidArg);
    }
    // SAFETY: `cfg` is non-null and `struct_size` covers this type.
    unsafe { *cfg }.to_config()
}

/// Reassembled access unit. `data`/`len` borrow session memory until the next
/// `lilynk_client_poll_frame` / `lilynk_session_free` on this session.
#[repr(C)]
pub struct LilynkFrame {
    pub data: *const u8,
    pub len: usize,
    pub frame_index: u32,
    pub pts_ns: u64,
    pub flags: u32,
    /// Reassembly-complete instant, ns since Unix epoch (`CLOCK_REALTIME`, same
    /// clock as `pts_ns`). A stamp at poll return includes pre-decode queue wait.
    pub received_ns: u64,
}

/// Session counters.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct LilynkStats {
    pub frames_submitted: u64,
    pub frames_completed: u64,
    pub frames_dropped: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packets_dropped: u64,
    /// Host send-path drops (`WouldBlock`). Distinct from recv-side `packets_dropped`.
    pub packets_send_dropped: u64,
    pub fec_recovered_shards: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl From<Stats> for LilynkStats {
    fn from(s: Stats) -> Self {
        LilynkStats {
            frames_submitted: s.frames_submitted,
            frames_completed: s.frames_completed,
            frames_dropped: s.frames_dropped,
            packets_sent: s.packets_sent,
            packets_received: s.packets_received,
            packets_dropped: s.packets_dropped,
            packets_send_dropped: s.packets_send_dropped,
            fec_recovered_shards: s.fec_recovered_shards,
            bytes_sent: s.bytes_sent,
            bytes_received: s.bytes_received,
        }
    }
}

fn new_handle(session: Session) -> *mut LilynkSession {
    Box::into_raw(Box::new(LilynkSession {
        inner: session,
        last_frame: None,
    }))
}

/// Connected host+client pair on in-process loopback. Test/dev only: full FEC
/// + framing without a network, unsealed.
///
/// # Safety
/// All four pointers are valid; the two out-params receive owned handles.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_test_loopback_pair(
    host_cfg: *const LilynkConfig,
    client_cfg: *const LilynkConfig,
    out_host: *mut *mut LilynkSession,
    out_client: *mut *mut LilynkSession,
) -> LilynkStatus {
    guard(|| {
        if host_cfg.is_null() || client_cfg.is_null() || out_host.is_null() || out_client.is_null()
        {
            return LilynkStatus::NullPointer;
        }
        // SAFETY: pointers are caller-supplied and null-checked on this path.
        let hconf = match unsafe { config_from_ptr(host_cfg) } {
            Ok(c) => c,
            Err(s) => return s,
        };
        // SAFETY: pointers are caller-supplied and null-checked on this path.
        let cconf = match unsafe { config_from_ptr(client_cfg) } {
            Ok(c) => c,
            Err(s) => return s,
        };
        let (ht, ct) = loopback_pair(hconf.loopback_drop_period, cconf.loopback_drop_period);
        let hs = match Session::new(hconf, MediaV2::default(), Box::new(ht)) {
            Ok(s) => s,
            Err(e) => return e.status(),
        };
        let cs = match Session::new(cconf, MediaV2::default(), Box::new(ct)) {
            Ok(s) => s,
            Err(e) => return e.status(),
        };
        // SAFETY: `out` is a caller-owned `#[repr(C)]` slot, written once by value.
        unsafe {
            *out_host = new_handle(hs);
            *out_client = new_handle(cs);
        }
        LilynkStatus::Ok
    })
}

/// Free a session handle. NULL is a no-op.
///
/// # Safety
/// `s` is a handle from `lilynk_test_loopback_pair`, freed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_session_free(s: *mut LilynkSession) {
    guard_void(|| {
        if !s.is_null() {
            // SAFETY: pointers are caller-supplied and null-checked on this path.
            drop(unsafe { Box::from_raw(s) });
        }
    });
}

/// Host: FEC-protect, packetize, seal, and send one encoded access unit.
///
/// # Safety
/// `s` is a valid host handle. For a representable nonzero `len`, `data` points
/// to that many readable bytes; `data` may be NULL when `len == 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_host_submit_frame(
    s: *mut LilynkSession,
    data: *const u8,
    len: usize,
    pts_ns: u64,
    flags: u32,
) -> LilynkStatus {
    guard(|| {
        // SAFETY: caller handle or null; `as_mut`/`as_ref` never dereference null.
        let s = match unsafe { s.as_mut() } {
            Some(s) => s,
            None => return LilynkStatus::NullPointer,
        };
        // SAFETY: `data` is null or readable for `len` bytes (this fn's contract).
        let slice = match unsafe { in_bytes(data, len) } {
            Ok(b) => b,
            Err(s) => return s,
        };
        status_of(s.inner.submit_frame(slice, pts_ns, flags))
    })
}

/// Client: poll for the next reassembled access unit. [`LilynkStatus::NoFrame`]
/// when nothing is ready. On `Ok`, `*out` borrows session memory until the next poll.
///
/// # Safety
/// `s` is a valid client handle; `out` points to a writable `LilynkFrame`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_client_poll_frame(
    s: *mut LilynkSession,
    out: *mut LilynkFrame,
) -> LilynkStatus {
    guard(|| {
        // SAFETY: caller handle or null; `as_mut`/`as_ref` never dereference null.
        let s = match unsafe { s.as_mut() } {
            Some(s) => s,
            None => return LilynkStatus::NullPointer,
        };
        if out.is_null() {
            return LilynkStatus::NullPointer;
        }
        match s.inner.poll_frame() {
            Ok(frame) => {
                let f = s.last_frame.insert(frame);
                // SAFETY: `out` is a caller-owned `#[repr(C)]` slot, written once by value.
                unsafe {
                    *out = LilynkFrame {
                        data: f.data.as_ptr(),
                        len: f.data.len(),
                        frame_index: f.frame_index,
                        pts_ns: f.pts_ns,
                        flags: f.flags,
                        received_ns: f.received_ns,
                    };
                }
                LilynkStatus::Ok
            }
            Err(e) => e.status(),
        }
    })
}

/// Validate the `kind` tag as a raw byte before forming `&InputEvent`. An
/// unknown `ev->kind` is UB once the typed reference exists; other fields are integers.
///
/// # Safety
/// `ev` is null (status) or readable for `size_of::<InputEvent>()` bytes.
#[cfg(feature = "quic")]
pub(crate) unsafe fn read_input_event<'a>(
    ev: *const InputEvent,
) -> Result<&'a InputEvent, LilynkStatus> {
    if ev.is_null() {
        return Err(LilynkStatus::NullPointer);
    }
    // SAFETY: non-null, readable; a one-byte read of the leading `kind` tag is valid for any value.
    if lilynk_core::input::InputKind::from_u8(unsafe { ev.cast::<u8>().read() }).is_none() {
        return Err(LilynkStatus::InvalidArg);
    }
    // SAFETY: discriminant validated; remaining fields are valid for any bit pattern.
    Ok(unsafe { &*ev })
}

/// Copy session counters into `*out`.
///
/// # Safety
/// `s` is a valid handle; `out` points to a writable `LilynkStats`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_get_stats(
    s: *mut LilynkSession,
    out: *mut LilynkStats,
) -> LilynkStatus {
    guard(|| {
        // SAFETY: caller handle or null; `as_mut`/`as_ref` never dereference null.
        let s = match unsafe { s.as_ref() } {
            Some(s) => s,
            None => return LilynkStatus::NullPointer,
        };
        if out.is_null() {
            return LilynkStatus::NullPointer;
        }
        let stats = s.inner.stats();
        // SAFETY: `out` is non-null on this path; written once by value.
        unsafe { *out = LilynkStats::from(stats) };
        LilynkStatus::Ok
    })
}

#[cfg(all(test, feature = "quic"))]
mod tests {
    use super::*;

    /// Invalid `kind` is a status, not UB. Staged in `MaybeUninit` so no `&InputEvent` to 42.
    #[test]
    fn read_input_event_rejects_null_and_bad_discriminant() {
        // SAFETY: null is the documented reported-not-UB case.
        let null_result = unsafe { read_input_event(std::ptr::null()) };
        assert_eq!(null_result.unwrap_err(), LilynkStatus::NullPointer);

        let mut slot = core::mem::MaybeUninit::<InputEvent>::zeroed();
        let p = slot.as_mut_ptr();
        // SAFETY: writing one byte at offset 0 of aligned, sized storage.
        unsafe { p.cast::<u8>().write(42) };
        // SAFETY: `p` is aligned and readable for the full struct.
        let bad_tag = unsafe { read_input_event(p) };
        assert_eq!(bad_tag.unwrap_err(), LilynkStatus::InvalidArg);

        // SAFETY: as above; tag 0 (KeyDown) + zeroed fields is a fully valid event.
        unsafe { p.cast::<u8>().write(0) };
        // SAFETY: as above.
        let ev = unsafe { read_input_event(p) }.expect("valid tag must pass");
        assert_eq!(ev.kind, lilynk_core::input::InputKind::KeyDown);
    }
}
