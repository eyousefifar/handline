// SPDX-License-Identifier: Apache-2.0

use crate::codec::{Rd, W};
use crate::error::ProtoError;
use crate::MAX_PLAINTEXT;

pub const HEADER_LEN: usize = 9;
pub const TAG_LEN: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameParts {
    pub version: u8,
    pub msg_type: u8,
    pub flags: u8,
    pub counter: u32,
    pub ciphertext: Vec<u8>,
    pub tag: [u8; TAG_LEN],
}

pub fn encode_frame(msg_type: u8, flags: u8, counter: u32, sealed: &[u8]) -> Result<Vec<u8>, ProtoError> {
    if sealed.len() < TAG_LEN {
        return Err(ProtoError::BadLength);
    }
    let ct_len = sealed.len() - TAG_LEN;
    if ct_len > MAX_PLAINTEXT {
        return Err(ProtoError::Oversized);
    }
    let mut w = W::new();
    w.u8(1);
    w.u8(msg_type);
    w.u8(flags);
    w.u32(counter);
    w.u16(ct_len as u16);
    w.raw(sealed);
    Ok(w.buf)
}

pub fn split_frame(bytes: &[u8]) -> Result<FrameParts, ProtoError> {
    if bytes.len() < HEADER_LEN + TAG_LEN {
        return Err(ProtoError::Truncated);
    }
    let mut r = Rd::new(bytes);
    let version = r.u8()?;
    if version != 1 {
        return Err(ProtoError::BadVersion);
    }
    let msg_type = r.u8()?;
    let flags = r.u8()?;
    let counter = r.u32()?;
    let ct_len = r.u16()? as usize;
    if ct_len > MAX_PLAINTEXT {
        return Err(ProtoError::Oversized);
    }
    let expected = HEADER_LEN + ct_len + TAG_LEN;
    if bytes.len() < expected {
        return Err(ProtoError::Truncated);
    }
    if bytes.len() != expected {
        return Err(ProtoError::BadLength);
    }
    let ciphertext = r.exact(ct_len)?.to_vec();
    let tag = r.arr()?;
    r.finish()?;
    Ok(FrameParts {
        version,
        msg_type,
        flags,
        counter,
        ciphertext,
        tag,
    })
}

/// AAD is version || type || flags || counter, all big-endian. The length field is not included.
pub fn aad(msg_type: u8, flags: u8, counter: u32) -> [u8; 7] {
    let c = counter.to_be_bytes();
    [1, msg_type, flags, c[0], c[1], c[2], c[3]]
}

/// Header-only check used by property tests. Decrypt is separate.
pub fn sniff(bytes: &[u8]) -> Result<(), ProtoError> {
    split_frame(bytes).map(|_| ())
}
