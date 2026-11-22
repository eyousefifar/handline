// SPDX-License-Identifier: Apache-2.0

//! Handline reference protocol.
//!
//! This crate is the behavior other implementations are tested against.
//! The wire format is documented in `docs/protocol.md`.

mod codec;
mod crypto;
mod error;
mod frame;
mod identity;
mod link;
mod msg;
mod qr;

pub use error::ProtoError;
pub use identity::{Identity, Role};
pub use link::{
    pair, pump, reconnect, CallPhase, CallView, Endpoint, HistoryView, LinkStatus, SmsView,
};
pub use frame::sniff as sniff_frame;
pub use qr::{decode_qr, BIND_SECONDS};
pub use msg::{decode_body as decode_body_for_test, is_otp, Message, AUDIO_MAX, SMS_BODY_MAX};
pub use qr::ParsedQr;

/// Sliding window of acceptable counters after the last in-order one.
pub const REPLAY_WINDOW: u32 = 32;

/// Plaintext cap inside one frame, bytes.
pub const MAX_PLAINTEXT: usize = 4096;

pub fn reference_fixture_json() -> String {
    link::reference_fixture_json()
}
