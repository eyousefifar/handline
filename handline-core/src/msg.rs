// SPDX-License-Identifier: Apache-2.0

use crate::codec::{Rd, W};
use crate::error::ProtoError;

pub const SMS_BODY_MAX: usize = 3500;
pub const AUDIO_MAX: usize = 512;
pub const PHOTO_MAX: usize = 2048;
pub const NAME_MAX: usize = 128;
pub const NUMBER_MAX: usize = 32;

pub const SIM_DEFAULT: u8 = 0xFF;

pub const DIR_INCOMING: u8 = 0;
pub const DIR_OUTGOING: u8 = 1;

pub const ST_RINGING: u8 = 0;
#[allow(dead_code)]
const _RINGING_IS_WIRE_ZERO: u8 = ST_RINGING;
pub const ST_ACTIVE: u8 = 1;
pub const ST_HELD: u8 = 2;
pub const ST_ENDED: u8 = 3;
pub const ST_DECLINED: u8 = 4;
pub const ST_BUSY: u8 = 5;
pub const ST_FAILED: u8 = 6;

pub const T_HELLO: u8 = 1;
pub const T_PING: u8 = 2;
pub const T_PONG: u8 = 3;
pub const T_CALL_INVITE: u8 = 4;
pub const T_CALL_STATE: u8 = 5;
pub const T_DTMF: u8 = 6;
pub const T_AUDIO: u8 = 7;
pub const T_SMS: u8 = 8;
pub const T_SMS_ACK: u8 = 9;
pub const T_CONTACT: u8 = 10;
pub const T_CONTACT_ACK: u8 = 11;
pub const T_HISTORY: u8 = 12;
pub const T_SIMS: u8 = 13;
pub const T_ERROR: u8 = 14;
pub const T_SYNC: u8 = 15;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub id: u64,
    pub timestamp_ms: u64,
    pub direction: u8,
    pub status: u8,
    pub sim: u8,
    pub number: String,
    pub name: String,
    pub duration_s: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimCard {
    pub slot: u8,
    pub label: String,
    pub number: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Hello { tier: u8, proto: u8, device_id: [u8; 16], name: String },
    Ping { ms: u64 },
    Pong { ms: u64 },
    CallInvite { direction: u8, call_id: u64, sim: u8, number: String, name: String },
    CallState { call_id: u64, state: u8, reason: u8 },
    Dtmf { call_id: u64, digit: u8 },
    Audio { call_id: u64, seq: u32, opus: Vec<u8> },
    Sms { id: u64, timestamp_ms: u64, folder: u8, sim: u8, address: String, body: String },
    SmsAck { id: u64, cursor: u64 },
    Contact { id: u64, op: u8, name: String, number: String, label: String, photo: Vec<u8> },
    ContactAck { id: u64, cursor: u64 },
    HistoryPage { cursor_from: u64, entries: Vec<HistoryEntry> },
    SimList { sims: Vec<SimCard> },
    ErrorMsg { code: u16, message: String },
    SyncCursor { collection: u8, cursor: u64 },
}

impl Message {
    pub fn type_byte(&self) -> u8 {
        match self {
            Self::Hello { .. } => T_HELLO,
            Self::Ping { .. } => T_PING,
            Self::Pong { .. } => T_PONG,
            Self::CallInvite { .. } => T_CALL_INVITE,
            Self::CallState { .. } => T_CALL_STATE,
            Self::Dtmf { .. } => T_DTMF,
            Self::Audio { .. } => T_AUDIO,
            Self::Sms { .. } => T_SMS,
            Self::SmsAck { .. } => T_SMS_ACK,
            Self::Contact { .. } => T_CONTACT,
            Self::ContactAck { .. } => T_CONTACT_ACK,
            Self::HistoryPage { .. } => T_HISTORY,
            Self::SimList { .. } => T_SIMS,
            Self::ErrorMsg { .. } => T_ERROR,
            Self::SyncCursor { .. } => T_SYNC,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtoError> {
        let mut w = W::new();
        match self {
            Self::Hello { tier, proto, device_id, name } => {
                w.u8(*tier);
                w.u8(*proto);
                w.raw(device_id);
                w.str(name, NAME_MAX)?;
            }
            Self::Ping { ms } | Self::Pong { ms } => w.u64(*ms),
            Self::CallInvite { direction, call_id, sim, number, name } => {
                w.u8(*direction);
                w.u64(*call_id);
                w.u8(*sim);
                w.str(number, NUMBER_MAX)?;
                w.str(name, NAME_MAX)?;
            }
            Self::CallState { call_id, state, reason } => {
                w.u64(*call_id);
                w.u8(*state);
                w.u8(*reason);
            }
            Self::Dtmf { call_id, digit } => {
                w.u64(*call_id);
                w.u8(*digit);
            }
            Self::Audio { call_id, seq, opus } => {
                w.u64(*call_id);
                w.u32(*seq);
                w.blob(opus, AUDIO_MAX)?;
            }
            Self::Sms { id, timestamp_ms, folder, sim, address, body } => {
                w.u64(*id);
                w.u64(*timestamp_ms);
                w.u8(*folder);
                w.u8(*sim);
                w.str(address, NUMBER_MAX)?;
                w.str(body, SMS_BODY_MAX)?;
            }
            Self::SmsAck { id, cursor } | Self::ContactAck { id, cursor } => {
                w.u64(*id);
                w.u64(*cursor);
            }
            Self::Contact { id, op, name, number, label, photo } => {
                w.u64(*id);
                w.u8(*op);
                w.str(name, NAME_MAX)?;
                w.str(number, NUMBER_MAX)?;
                w.str(label, NAME_MAX)?;
                w.blob(photo, PHOTO_MAX)?;
            }
            Self::HistoryPage { cursor_from, entries } => {
                if entries.len() > 64 {
                    return Err(ProtoError::Oversized);
                }
                w.u64(*cursor_from);
                w.u16(entries.len() as u16);
                for e in entries {
                    w.u64(e.id);
                    w.u64(e.timestamp_ms);
                    w.u8(e.direction);
                    w.u8(e.status);
                    w.u8(e.sim);
                    w.str(&e.number, NUMBER_MAX)?;
                    w.str(&e.name, NAME_MAX)?;
                    w.u32(e.duration_s);
                }
            }
            Self::SimList { sims } => {
                if sims.len() > 8 {
                    return Err(ProtoError::Oversized);
                }
                w.u8(sims.len() as u8);
                for s in sims {
                    w.u8(s.slot);
                    w.str(&s.label, NAME_MAX)?;
                    w.str(&s.number, NUMBER_MAX)?;
                }
            }
            Self::ErrorMsg { code, message } => {
                w.u16(*code);
                w.str(message, NAME_MAX)?;
            }
            Self::SyncCursor { collection, cursor } => {
                w.u8(*collection);
                w.u64(*cursor);
            }
        }
        if w.buf.len() > crate::MAX_PLAINTEXT {
            return Err(ProtoError::Oversized);
        }
        Ok(w.buf)
    }
}

pub fn decode_body(msg_type: u8, plain: &[u8]) -> Result<Message, ProtoError> {
    let mut r = Rd::new(plain);
    let msg = match msg_type {
        T_HELLO => Message::Hello {
            tier: r.u8()?,
            proto: r.u8()?,
            device_id: r.arr()?,
            name: r.str(NAME_MAX)?,
        },
        T_PING => Message::Ping { ms: r.u64()? },
        T_PONG => Message::Pong { ms: r.u64()? },
        T_CALL_INVITE => Message::CallInvite {
            direction: r.u8()?,
            call_id: r.u64()?,
            sim: r.u8()?,
            number: r.str(NUMBER_MAX)?,
            name: r.str(NAME_MAX)?,
        },
        T_CALL_STATE => Message::CallState {
            call_id: r.u64()?,
            state: r.u8()?,
            reason: r.u8()?,
        },
        T_DTMF => Message::Dtmf {
            call_id: r.u64()?,
            digit: r.u8()?,
        },
        T_AUDIO => Message::Audio {
            call_id: r.u64()?,
            seq: r.u32()?,
            opus: r.blob(AUDIO_MAX)?,
        },
        T_SMS => Message::Sms {
            id: r.u64()?,
            timestamp_ms: r.u64()?,
            folder: r.u8()?,
            sim: r.u8()?,
            address: r.str(NUMBER_MAX)?,
            body: r.str(SMS_BODY_MAX)?,
        },
        T_SMS_ACK => Message::SmsAck { id: r.u64()?, cursor: r.u64()? },
        T_CONTACT => Message::Contact {
            id: r.u64()?,
            op: r.u8()?,
            name: r.str(NAME_MAX)?,
            number: r.str(NUMBER_MAX)?,
            label: r.str(NAME_MAX)?,
            photo: r.blob(PHOTO_MAX)?,
        },
        T_CONTACT_ACK => Message::ContactAck { id: r.u64()?, cursor: r.u64()? },
        T_HISTORY => {
            let cursor_from = r.u64()?;
            let n = r.u16()? as usize;
            if n > 64 {
                return Err(ProtoError::Oversized);
            }
            let mut entries = Vec::with_capacity(n);
            for _ in 0..n {
                entries.push(HistoryEntry {
                    id: r.u64()?,
                    timestamp_ms: r.u64()?,
                    direction: r.u8()?,
                    status: r.u8()?,
                    sim: r.u8()?,
                    number: r.str(NUMBER_MAX)?,
                    name: r.str(NAME_MAX)?,
                    duration_s: r.u32()?,
                });
            }
            Message::HistoryPage { cursor_from, entries }
        }
        T_SIMS => {
            let n = r.u8()? as usize;
            if n > 8 {
                return Err(ProtoError::Oversized);
            }
            let mut sims = Vec::with_capacity(n);
            for _ in 0..n {
                sims.push(SimCard {
                    slot: r.u8()?,
                    label: r.str(NAME_MAX)?,
                    number: r.str(NUMBER_MAX)?,
                });
            }
            Message::SimList { sims }
        }
        T_ERROR => Message::ErrorMsg {
            code: r.u16()?,
            message: r.str(NAME_MAX)?,
        },
        T_SYNC => Message::SyncCursor {
            collection: r.u8()?,
            cursor: r.u64()?,
        },
        _ => return Err(ProtoError::UnknownType),
    };
    r.finish()?;
    Ok(msg)
}

pub fn is_otp(body: &str) -> bool {
    let trimmed = body.trim();
    if (4..=8).contains(&trimmed.len()) && trimmed.bytes().all(|b| b.is_ascii_digit()) {
        return true;
    }
    let lower = body.to_ascii_lowercase();
    let keyword = ["code", "otp", "verify", "verification"]
        .iter()
        .any(|k| lower.contains(k));
    keyword && digit_run(body, 4, 8)
}

fn digit_run(body: &str, min: usize, max: usize) -> bool {
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let n = i - start;
            if (min..=max).contains(&n) {
                return true;
            }
        } else {
            i += 1;
        }
    }
    false
}

pub fn valid_dtmf(digit: u8) -> bool {
    matches!(digit, b'0'..=b'9' | b'*' | b'#' | b'A'..=b'D')
}
