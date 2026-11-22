// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ProtoError {
    #[error("truncated")]
    Truncated,
    #[error("bad_magic")]
    BadMagic,
    #[error("bad_version")]
    BadVersion,
    #[error("bad_length")]
    BadLength,
    #[error("bad_public_key")]
    BadPublicKey,
    #[error("bad_utf8")]
    BadUtf8,
    #[error("oversized")]
    Oversized,
    #[error("expired_qr")]
    ExpiredQr,
    #[error("reused_token")]
    ReusedToken,
    #[error("no_offer")]
    NoOffer,
    #[error("not_paired")]
    NotPaired,
    #[error("finished_mismatch")]
    FinishedMismatch,
    #[error("replay")]
    Replay,
    #[error("counter_ahead")]
    CounterAhead,
    #[error("bad_state")]
    BadState,
    #[error("audio_unavailable")]
    AudioUnavailable,
    #[error("unknown_type")]
    UnknownType,
    #[error("unexpected_flags")]
    UnexpectedFlags,
    #[error("hello_required")]
    HelloRequired,
    #[error("session_down")]
    SessionDown,
    #[error("wrong_role")]
    WrongRole,
    #[error("crypto")]
    Crypto,
    #[error("bad_digit")]
    BadDigit,
}

impl ProtoError {
    pub fn code_name(self) -> &'static str {
        match self {
            Self::Truncated => "truncated",
            Self::BadMagic => "bad_magic",
            Self::BadVersion => "bad_version",
            Self::BadLength => "bad_length",
            Self::BadPublicKey => "bad_public_key",
            Self::BadUtf8 => "bad_utf8",
            Self::Oversized => "oversized",
            Self::ExpiredQr => "expired_qr",
            Self::ReusedToken => "reused_token",
            Self::NoOffer => "no_offer",
            Self::NotPaired => "not_paired",
            Self::FinishedMismatch => "finished_mismatch",
            Self::Replay => "replay",
            Self::CounterAhead => "counter_ahead",
            Self::BadState => "bad_state",
            Self::AudioUnavailable => "audio_unavailable",
            Self::UnknownType => "unknown_type",
            Self::UnexpectedFlags => "unexpected_flags",
            Self::HelloRequired => "hello_required",
            Self::SessionDown => "session_down",
            Self::WrongRole => "wrong_role",
            Self::Crypto => "crypto",
            Self::BadDigit => "bad_digit",
        }
    }
}
