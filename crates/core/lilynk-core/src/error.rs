//! Core error type and the C ABI status codes it maps to.
//!
//! `LilynkStatus` is stable: `Ok` is 0, errors are negative, existing
//! variants must not be renumbered. Rejection codes live in the -20 block.

use thiserror::Error;

/// Internal error. Crosses the C ABI as a [`LilynkStatus`] code.
#[derive(Debug, Error)]
pub enum LilynkError {
    #[error("invalid argument: {0}")]
    InvalidArg(&'static str),
    #[error("fec error: {0}")]
    Fec(#[from] crate::fec::FecError),
    #[error("crypto seal/open failed")]
    Crypto,
    #[error("malformed packet")]
    BadPacket,
    #[error("no complete frame available yet")]
    NoFrame,
    #[error("unsupported: {0}")]
    Unsupported(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("timed out")]
    Timeout,
    #[error("session closed")]
    Closed,
    /// Host turned this connection away with a typed
    /// [`crate::reject::RejectReason`], and the sentence it sent with it: empty when it had
    /// none, and already sanitized. Distinct from transport ([`Self::Io`] / [`Self::Timeout`])
    /// and a failed PIN ([`Self::Crypto`]).
    #[error("rejected by host: {0}")]
    Rejected(crate::reject::RejectReason, String),
}

pub type Result<T> = core::result::Result<T, LilynkError>;

/// Stable C ABI status codes. `Ok` is 0; errors are negative so callers can
/// test `rc < 0`. Existing variants must not be renumbered — only append.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LilynkStatus {
    Ok = 0,
    InvalidArg = -1,
    Fec = -2,
    Crypto = -3,
    BadPacket = -4,
    NoFrame = -5,
    Unsupported = -6,
    Io = -7,
    NullPointer = -8,
    Timeout = -9,
    Closed = -10,
    /// `next_audio_pcm`: the host changed the audio format at the next packet. Reopen the player
    /// from the audio getters, which read the new format, then pull again.
    AudioFormat = -11,
    // -12..-19 reserved. The -20 block mirrors `RejectReason` one-to-one
    // so FFI callers can switch on the host's reason.
    RejectedNotArmed = -20,
    RejectedBoundOther = -21,
    RejectedRateLimited = -22,
    RejectedIdentityRequired = -23,
    RejectedDenied = -24,
    RejectedApprovalTimeout = -25,
    RejectedSuperseded = -26,
    RejectedWireVersion = -27,
    RejectedBusy = -28,
    RejectedSetupFailed = -29,
    RejectedAccessExpired = -30,
    RejectedLaunchNotPermitted = -31,
    RejectedHostPower = -32,
    RejectedProfileUnknown = -33,
    RejectedNoSeat = -34,
    RejectedSeatOccupied = -35,
    RejectedSeatUnavailable = -36,
    RejectedProfileLocked = -37,
    Panic = -99,
}

impl LilynkError {
    pub fn status(&self) -> LilynkStatus {
        match self {
            LilynkError::InvalidArg(_) => LilynkStatus::InvalidArg,
            LilynkError::Fec(_) => LilynkStatus::Fec,
            LilynkError::Crypto => LilynkStatus::Crypto,
            LilynkError::BadPacket => LilynkStatus::BadPacket,
            LilynkError::NoFrame => LilynkStatus::NoFrame,
            LilynkError::Unsupported(_) => LilynkStatus::Unsupported,
            LilynkError::Io(_) => LilynkStatus::Io,
            LilynkError::Timeout => LilynkStatus::Timeout,
            LilynkError::Closed => LilynkStatus::Closed,
            LilynkError::Rejected(r, _) => {
                use crate::reject::RejectReason as R;
                match r {
                    R::PairingNotArmed => LilynkStatus::RejectedNotArmed,
                    R::PairingBoundToOtherDevice => LilynkStatus::RejectedBoundOther,
                    R::PairingRateLimited => LilynkStatus::RejectedRateLimited,
                    R::IdentityRequired => LilynkStatus::RejectedIdentityRequired,
                    R::Denied => LilynkStatus::RejectedDenied,
                    R::ApprovalTimeout => LilynkStatus::RejectedApprovalTimeout,
                    R::Superseded => LilynkStatus::RejectedSuperseded,
                    R::WireVersionMismatch => LilynkStatus::RejectedWireVersion,
                    R::Busy => LilynkStatus::RejectedBusy,
                    R::SetupFailed => LilynkStatus::RejectedSetupFailed,
                    R::AccessExpired => LilynkStatus::RejectedAccessExpired,
                    R::LaunchNotPermitted => LilynkStatus::RejectedLaunchNotPermitted,
                    R::HostPower => LilynkStatus::RejectedHostPower,
                    R::ProfileUnknown => LilynkStatus::RejectedProfileUnknown,
                    R::NoSeat => LilynkStatus::RejectedNoSeat,
                    R::SeatOccupied => LilynkStatus::RejectedSeatOccupied,
                    R::SeatUnavailable => LilynkStatus::RejectedSeatUnavailable,
                    R::ProfileLocked => LilynkStatus::RejectedProfileLocked,
                }
            }
        }
    }
}
