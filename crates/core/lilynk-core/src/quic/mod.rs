//! The control plane's message vocabulary, and the quinn transport that carries it.
//!
//! **The messages are not behind the `quic` feature; the transport is.** A browser speaks the
//! same protocol over WebTransport, where quinn cannot go, so only [`endpoint`], [`lyk1`],
//! [`clipstream`] and [`pake`] need a feature.
//!
//! The structs here are the session's semantic model; [`v2`] encodes them for the wire (ALPN
//! `lyk2`). The handshake is `ClientHello` → `ServerHello` → `Ready` on the first stream, then
//! both sides open a [`crate::session::Session`] over the connection's own socket, keyed from
//! its TLS exporter. The host presents a long-lived self-signed cert; the client pins its
//! SHA-256 fingerprint (no pin = TOFU).

/// First bytes of a [`lyk1`] pairing message or refusal.
pub const CTL_MAGIC: &[u8; 4] = b"LYKc";

mod access;
mod caps;
mod clock;
mod control;
mod datagram;
mod handshake;
mod pairing;
mod pen;
/// The C surface is `lilynk-ffi`'s own touchpad types, not these.
/// cbindgen:ignore
mod touchpad;
mod wire;

/// The `lilynk/2` codecs (ALPN `lyk2`). Ungated like the v1 vocabulary: the browser speaks it too.
/// cbindgen:ignore
pub mod v2;

/// quinn endpoint constructors: the host's shared socket ([`endpoint::server_shared`]),
/// client pin / TOFU ([`endpoint::client_pinned`]).
#[cfg(feature = "quic")]
pub mod endpoint;

/// The PIN ceremony and refusal an older client still speaks over `lyk1`.
#[cfg(feature = "quic")]
pub mod lyk1;

/// Per-transfer clipboard fetch streams: a transfer stream, a request frame, then raw bytes.
/// Transport only; state per side.
#[cfg(feature = "quic")]
pub mod clipstream;

/// SPAKE2 over Ed25519 for pairing. Both certificate fingerprints are the SPAKE2
/// identities, so a MITM that presents different certs on each leg cannot share a key.
/// Its own feature, so a client that pairs over a different transport can have the ceremony
/// without quinn.
#[cfg(feature = "pake")]
pub mod pake;

pub use access::*;
pub use caps::*;
pub use clock::*;
pub use control::*;
pub use datagram::*;
pub use handshake::*;
pub use pairing::*;
pub use pen::*;
pub use touchpad::*;

// Close codes + [`RejectReason`] live in `crate::reject` (ungated: the error enum
// names them even without `quic`) and re-export here next to QUIT/APP_EXITED.
pub use crate::reject::*;

// `quic` as well as `test`: this hands out `quinn` endpoints, and `endpoint` above is
// feature-gated. Gated on `test` alone, `cargo test -p lilynk-core` would not build:
// it resolves default features, which have no quinn.
#[cfg(all(test, feature = "quic"))]
pub(crate) mod test_util;
