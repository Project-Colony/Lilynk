//! ScreenCast cursor mode. The ladder is [`ly_portal::cursor_mode`], shared with
//! `ly-capture`'s portal monitor; the Linux negotiation against
//! `AvailableCursorModes` is `ly_portal::negotiate_cursor_mode`.

pub use ly_portal::cursor_mode::Mode;
#[cfg(target_os = "linux")]
pub(crate) use ly_portal::negotiate_cursor_mode as negotiate;
