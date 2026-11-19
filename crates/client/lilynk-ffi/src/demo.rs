//! Demo host: a loopback host the embedder feeds for its demo mode (`crate::demo_host`).

#[cfg(feature = "quic")]
use crate::*;
#[cfg(feature = "quic")]
use lilynk_core::input::InputEvent;

/// A running demo host from [`lilynk_demo_host_start`].
#[cfg(feature = "quic")]
pub struct LilynkDemoHost {
    inner: crate::demo_host::DemoHost,
}

/// The demo session to render, from [`lilynk_demo_host_session`].
#[cfg(feature = "quic")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LilynkDemoSession {
    /// Bumped on each new session and accepted mode switch: rebuild the encoder, start on an IDR.
    pub generation: u32,
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub bitrate_kbps: u32,
    /// `LILYNK_CODEC_H264` or `LILYNK_CODEC_HEVC`.
    pub codec: u8,
}

/// Start a demo host on a free loopback port. `codecs` is the `LILYNK_CODEC_*` mask the
/// embedder can encode. NULL on failure. Free with [`lilynk_demo_host_stop`].
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub extern "C" fn lilynk_demo_host_start(codecs: u8) -> *mut LilynkDemoHost {
    let started = std::panic::catch_unwind(|| crate::demo_host::DemoHost::start(codecs));
    match started {
        Ok(Ok(inner)) => Box::into_raw(Box::new(LilynkDemoHost { inner })),
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "demo host did not start");
            ptr::null_mut()
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Stop the host, ending any session, and free the handle. NULL is a no-op.
///
/// # Safety
/// `h` came from [`lilynk_demo_host_start`] and is not used after this call.
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_demo_host_stop(h: *mut LilynkDemoHost) {
    guard_void(|| {
        if !h.is_null() {
            // SAFETY: pointers are caller-supplied and null-checked on this path.
            drop(unsafe { Box::from_raw(h) });
        }
    });
}

/// The loopback UDP port the host listens on. 0 for NULL.
///
/// # Safety
/// `h` is a live handle or NULL.
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_demo_host_port(h: *const LilynkDemoHost) -> u16 {
    // SAFETY: caller handle or null; `as_ref` never dereferences null.
    unsafe { h.as_ref() }.map_or(0, |h| h.inner.port())
}

/// Write the host certificate's SHA-256 to `out_sha256`: the pin to dial it with.
///
/// # Safety
/// `h` is a live handle; `out_sha256` is writable for 32 bytes.
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_demo_host_fingerprint(
    h: *const LilynkDemoHost,
    out_sha256: *mut u8,
) -> LilynkStatus {
    guard(|| {
        // SAFETY: caller handle or null; `as_ref` never dereferences null.
        let Some(h) = (unsafe { h.as_ref() }) else {
            return LilynkStatus::NullPointer;
        };
        if out_sha256.is_null() {
            return LilynkStatus::NullPointer;
        }
        // SAFETY: `out_sha256` is non-null here and writable for 32 bytes.
        unsafe { put_sha256(out_sha256, h.inner.fingerprint()) };
        LilynkStatus::Ok
    })
}

/// Write the live session to `out` and return true. False, with `out` untouched, while no
/// client is streaming.
///
/// # Safety
/// `h` is a live handle; `out` is writable.
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_demo_host_session(
    h: *const LilynkDemoHost,
    out: *mut LilynkDemoSession,
) -> bool {
    std::panic::catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: caller handle or null; `as_ref` never dereferences null.
        let Some(h) = (unsafe { h.as_ref() }) else {
            return false;
        };
        let Some(s) = h.inner.session().filter(|_| !out.is_null()) else {
            return false;
        };
        // SAFETY: caller out-param, non-null on this path, written once.
        unsafe {
            *out = LilynkDemoSession {
                generation: s.generation,
                width: s.mode.width,
                height: s.mode.height,
                refresh_hz: s.mode.refresh_hz,
                bitrate_kbps: s.bitrate_kbps,
                codec: s.codec,
            }
        };
        true
    }))
    .unwrap_or(false)
}

/// Copy the live session's launch id, NUL-terminated, into `buf`. Returns its length without
/// the NUL; 0 when no session or no launch. Writes nothing when `cap` < length + 1.
///
/// # Safety
/// `h` is a live handle; `buf` is writable for `cap` bytes.
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_demo_host_launch(
    h: *const LilynkDemoHost,
    buf: *mut c_char,
    cap: usize,
) -> usize {
    std::panic::catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: caller handle or null; `as_ref` never dereferences null.
        let Some(h) = (unsafe { h.as_ref() }) else {
            return 0;
        };
        let Some(launch) = h.inner.session().and_then(|s| s.launch) else {
            return 0;
        };
        // SAFETY: `buf` is null or writable for `cap` bytes, per this function's contract.
        unsafe { write_cstr(buf, cap, &launch) };
        launch.len()
    }))
    .unwrap_or(0)
}

/// True once per client keyframe request or dropped AU: encode the next frame as an IDR.
///
/// # Safety
/// `h` is a live handle or NULL.
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_demo_host_take_keyframe_request(h: *const LilynkDemoHost) -> bool {
    // SAFETY: caller handle or null; `as_ref` never dereferences null.
    unsafe { h.as_ref() }.is_some_and(|h| h.inner.take_keyframe_request())
}

/// Pop the oldest input event the client sent into `out`. False when none is waiting.
///
/// # Safety
/// `h` is a live handle; `out` is writable.
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_demo_host_next_input(
    h: *const LilynkDemoHost,
    out: *mut InputEvent,
) -> bool {
    std::panic::catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: caller handle or null; `as_ref` never dereferences null.
        let Some(h) = (unsafe { h.as_ref() }) else {
            return false;
        };
        if out.is_null() {
            return false;
        }
        let Some(ev) = h.inner.next_input() else {
            return false;
        };
        // SAFETY: caller out-param, non-null on this path, written once.
        unsafe { *out = ev };
        true
    }))
    .unwrap_or(false)
}

/// Queue one Annex-B access unit, parameter sets in-band on an IDR. False: no session, or the
/// queue was full and the AU dropped — make the next frame an IDR.
///
/// # Safety
/// `h` is a live handle; `data` is readable for `len` bytes.
#[cfg(feature = "quic")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lilynk_demo_host_submit_video(
    h: *const LilynkDemoHost,
    data: *const u8,
    len: usize,
    keyframe: bool,
) -> bool {
    std::panic::catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: caller handle or null; `as_ref` never dereferences null.
        let Some(h) = (unsafe { h.as_ref() }) else {
            return false;
        };
        if data.is_null() || ffi_slice_bytes::<u8>(len).is_none() {
            return false;
        }
        // SAFETY: the caller guarantees `len` readable bytes at non-null `data`.
        let au = unsafe { std::slice::from_raw_parts(data, len) };
        h.inner.submit_video(au, keyframe)
    }))
    .unwrap_or(false)
}
