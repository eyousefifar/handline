// SPDX-License-Identifier: Apache-2.0

use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use hkdf::Hkdf;
use p256::ecdh::diffie_hellman;
use p256::{PublicKey, SecretKey};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::error::ProtoError;
use crate::frame::{self, FrameParts};
use crate::identity::parse_compressed;
use crate::msg::{decode_body, Message};
use crate::REPLAY_WINDOW;

pub fn ecdh(secret: &SecretKey, peer: &PublicKey) -> [u8; 32] {
    let shared = diffie_hellman(secret.to_nonzero_scalar(), peer.as_affine());
    let raw = shared.raw_secret_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(raw);
    out
}

pub fn derive_keys(
    dh1: &[u8; 32],
    dh2: &[u8; 32],
    dh3: &[u8; 32],
    dh4: &[u8; 32],
    nonce_i: &[u8; 16],
    nonce_b: &[u8; 16],
    token: &[u8; 16],
) -> ([u8; 32], [u8; 32]) {
    let mut ikm = [0u8; 128];
    ikm[0..32].copy_from_slice(dh1);
    ikm[32..64].copy_from_slice(dh2);
    ikm[64..96].copy_from_slice(dh3);
    ikm[96..128].copy_from_slice(dh4);
    let mut salt = [0u8; 48];
    salt[0..16].copy_from_slice(nonce_i);
    salt[16..32].copy_from_slice(nonce_b);
    salt[32..48].copy_from_slice(token);
    let hk = Hkdf::<Sha256>::new(Some(&salt), &ikm);
    let mut okm = [0u8; 64];
    hk.expand(b"handline v1", &mut okm)
        .expect("64-byte HKDF expand fits SHA-256");
    let mut c2s = [0u8; 32];
    let mut s2c = [0u8; 32];
    c2s.copy_from_slice(&okm[0..32]);
    s2c.copy_from_slice(&okm[32..64]);
    (c2s, s2c)
}

pub fn transcript_hash(client_hello: &[u8], server_hello: &[u8], token: &[u8; 16]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(client_hello);
    h.update(server_hello);
    h.update(token);
    let dig = h.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&dig);
    out
}

pub fn nonce(dir: u8, counter: u32) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[0] = dir;
    n[8..12].copy_from_slice(&counter.to_be_bytes());
    n
}

pub fn seal(key: &[u8; 32], nonce_bytes: &[u8; 12], aad: &[u8], plain: &[u8]) -> Result<Vec<u8>, ProtoError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| ProtoError::Crypto)?;
    cipher
        .encrypt(
            Nonce::from_slice(nonce_bytes),
            Payload { msg: plain, aad },
        )
        .map_err(|_| ProtoError::Crypto)
}

pub fn open(key: &[u8; 32], nonce_bytes: &[u8; 12], aad: &[u8], sealed: &[u8]) -> Result<Vec<u8>, ProtoError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| ProtoError::Crypto)?;
    cipher
        .decrypt(
            Nonce::from_slice(nonce_bytes),
            Payload { msg: sealed, aad },
        )
        .map_err(|_| ProtoError::Crypto)
}

pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    bool::from(a.ct_eq(b))
}

pub fn encode_record(kind: u8, payload: &[u8]) -> Result<Vec<u8>, ProtoError> {
    if payload.len() > u16::MAX as usize {
        return Err(ProtoError::Oversized);
    }
    let mut out = Vec::with_capacity(3 + payload.len());
    out.push(kind);
    out.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

pub fn decode_record(bytes: &[u8]) -> Result<(u8, Vec<u8>), ProtoError> {
    if bytes.len() < 3 {
        return Err(ProtoError::Truncated);
    }
    let kind = bytes[0];
    let len = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
    if bytes.len() < 3 + len {
        return Err(ProtoError::Truncated);
    }
    if bytes.len() != 3 + len {
        return Err(ProtoError::BadLength);
    }
    Ok((kind, bytes[3..].to_vec()))
}

pub struct ClientHello {
    pub eph: [u8; 33],
    pub static_key: [u8; 33],
    pub nonce_i: [u8; 16],
}

pub fn parse_client_hello(payload: &[u8]) -> Result<ClientHello, ProtoError> {
    if payload.len() != 82 {
        return Err(if payload.len() < 82 {
            ProtoError::Truncated
        } else {
            ProtoError::BadLength
        });
    }
    let mut eph = [0u8; 33];
    let mut static_key = [0u8; 33];
    let mut nonce_i = [0u8; 16];
    eph.copy_from_slice(&payload[0..33]);
    static_key.copy_from_slice(&payload[33..66]);
    nonce_i.copy_from_slice(&payload[66..82]);
    let _ = parse_compressed(&eph)?;
    let _ = parse_compressed(&static_key)?;
    Ok(ClientHello { eph, static_key, nonce_i })
}

pub fn parse_server_hello(payload: &[u8]) -> Result<([u8; 33], [u8; 16]), ProtoError> {
    if payload.len() != 49 {
        return Err(if payload.len() < 49 {
            ProtoError::Truncated
        } else {
            ProtoError::BadLength
        });
    }
    let mut eph = [0u8; 33];
    let mut nonce_b = [0u8; 16];
    eph.copy_from_slice(&payload[0..33]);
    nonce_b.copy_from_slice(&payload[33..49]);
    let _ = parse_compressed(&eph)?;
    Ok((eph, nonce_b))
}

#[derive(Clone)]
pub struct ReplayWindow {
    next: u32,
    bitmap: u64,
}

impl ReplayWindow {
    pub fn new() -> Self {
        Self { next: 1, bitmap: 0 }
    }

    pub fn check(&self, counter: u32) -> Result<(), ProtoError> {
        if counter == 0 || counter < self.next {
            return Err(ProtoError::Replay);
        }
        let off = counter - self.next;
        if off >= REPLAY_WINDOW {
            return Err(ProtoError::CounterAhead);
        }
        let bit = 1u64 << off;
        if self.bitmap & bit != 0 {
            return Err(ProtoError::Replay);
        }
        Ok(())
    }

    pub fn commit(&mut self, counter: u32) {
        let off = counter - self.next;
        self.bitmap |= 1u64 << off;
        while self.bitmap & 1 == 1 {
            self.bitmap >>= 1;
            self.next = self.next.saturating_add(1);
        }
    }
}

impl Default for ReplayWindow {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Session {
    send_key: [u8; 32],
    recv_key: [u8; 32],
    send_dir: u8,
    recv_dir: u8,
    send_counter: u32,
    recv: ReplayWindow,
    c2s: [u8; 32],
    s2c: [u8; 32],
}

impl Session {
    pub fn new(role_is_hand: bool, c2s: [u8; 32], s2c: [u8; 32]) -> Self {
        let (send_key, recv_key, send_dir, recv_dir) = if role_is_hand {
            (c2s, s2c, 1, 2)
        } else {
            (s2c, c2s, 2, 1)
        };
        Self {
            send_key,
            recv_key,
            send_dir,
            recv_dir,
            send_counter: 1,
            recv: ReplayWindow::new(),
            c2s,
            s2c,
        }
    }

    pub fn c2s(&self) -> [u8; 32] {
        self.c2s
    }

    pub fn s2c(&self) -> [u8; 32] {
        self.s2c
    }

    pub fn seal(&mut self, msg: &Message, flags: u8) -> Result<Vec<u8>, ProtoError> {
        if flags & !0x01 != 0 {
            return Err(ProtoError::UnexpectedFlags);
        }
        let counter = self.send_counter;
        let plain = msg.encode()?;
        let n = nonce(self.send_dir, counter);
        let aad = frame::aad(msg.type_byte(), flags, counter);
        let sealed = seal(&self.send_key, &n, &aad, &plain)?;
        let frame = frame::encode_frame(msg.type_byte(), flags, counter, &sealed)?;
        self.send_counter = self.send_counter.saturating_add(1);
        Ok(frame)
    }

    pub fn open(&mut self, bytes: &[u8]) -> Result<Message, ProtoError> {
        let parts = frame::split_frame(bytes)?;
        self.open_parts(&parts)
    }

    fn open_parts(&mut self, parts: &FrameParts) -> Result<Message, ProtoError> {
        let mut sealed = parts.ciphertext.clone();
        sealed.extend_from_slice(&parts.tag);
        let n = nonce(self.recv_dir, parts.counter);
        let aad = frame::aad(parts.msg_type, parts.flags, parts.counter);
        let plain = open(&self.recv_key, &n, &aad, &sealed)?;
        if parts.flags & !0x01 != 0 {
            return Err(ProtoError::UnexpectedFlags);
        }
        self.recv.check(parts.counter)?;
        let msg = decode_body(parts.msg_type, &plain)?;
        self.recv.commit(parts.counter);
        Ok(msg)
    }
}

pub fn finished_blob(key: &[u8; 32], dir: u8, transcript: &[u8; 32], aad: &'static [u8]) -> Result<Vec<u8>, ProtoError> {
    let n = nonce(dir, 0);
    seal(key, &n, aad, transcript)
}

pub fn finished_open(key: &[u8; 32], dir: u8, blob: &[u8], transcript: &[u8; 32], aad: &'static [u8]) -> Result<(), ProtoError> {
    let n = nonce(dir, 0);
    let plain = open(key, &n, aad, blob)?;
    if !ct_eq(&plain, transcript) {
        return Err(ProtoError::FinishedMismatch);
    }
    Ok(())
}
