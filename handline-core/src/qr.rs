// SPDX-License-Identifier: Apache-2.0

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

use crate::codec::{Rd, W};
use crate::error::ProtoError;
use crate::identity::Identity;

pub const QR_PREFIX: &str = "handline:";
pub const BIND_SECONDS: u64 = 300;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedQr {
    pub device_id: [u8; 16],
    pub public_key: [u8; 33],
    pub expiry: u64,
    pub token: [u8; 16],
    pub psm: u16,
    pub name: String,
}

pub fn encode_qr(identity: &Identity, expiry: u64, token: [u8; 16], psm: u16) -> Result<String, ProtoError> {
    let name = identity.name();
    if name.len() > 64 {
        return Err(ProtoError::Oversized);
    }
    let mut w = W::new();
    w.raw(b"HL01");
    w.u8(1);
    w.raw(&identity.device_id());
    w.raw(&identity.public_compressed());
    w.u64(expiry);
    w.raw(&token);
    w.u16(psm);
    w.u8(name.len() as u8);
    w.raw(name.as_bytes());
    Ok(format!("{QR_PREFIX}{}", URL_SAFE_NO_PAD.encode(w.buf)))
}

pub fn decode_qr(text: &str) -> Result<ParsedQr, ProtoError> {
    let rest = text.strip_prefix(QR_PREFIX).ok_or(ProtoError::BadMagic)?;
    let raw = URL_SAFE_NO_PAD
        .decode(rest.trim())
        .map_err(|_| ProtoError::BadMagic)?;
    let mut r = Rd::new(&raw);
    let magic: [u8; 4] = r.arr()?;
    if &magic != b"HL01" {
        return Err(ProtoError::BadMagic);
    }
    let version = r.u8()?;
    if version != 1 {
        return Err(ProtoError::BadVersion);
    }
    let device_id = r.arr()?;
    let public_key = r.arr()?;
    let _ = crate::identity::parse_compressed(&public_key)?;
    let expiry = r.u64()?;
    let token = r.arr()?;
    let psm = r.u16()?;
    let name_len = r.u8()? as usize;
    if name_len > 64 {
        return Err(ProtoError::Oversized);
    }
    let name_bytes = r.exact(name_len)?;
    let name = std::str::from_utf8(name_bytes)
        .map_err(|_| ProtoError::BadUtf8)?
        .to_string();
    r.finish()?;
    Ok(ParsedQr {
        device_id,
        public_key,
        expiry,
        token,
        psm,
        name,
    })
}
