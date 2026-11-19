//! The Apple client's one static library: the lilynk C ABI, linked whole, plus the
//! console's (`lilynk_console_*`). Swift imports both as `LilynkCore`.

pub use lilynk_ffi;

#[cfg(target_vendor = "apple")]
mod console;
