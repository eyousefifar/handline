// SPDX-License-Identifier: Apache-2.0

use crate::error::ProtoError;

pub struct W {
    pub buf: Vec<u8>,
}

impl W {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn raw(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }

    pub fn str(&mut self, s: &str, max: usize) -> Result<(), ProtoError> {
        let b = s.as_bytes();
        if b.len() > max {
            return Err(ProtoError::Oversized);
        }
        self.u16(b.len() as u16);
        self.buf.extend_from_slice(b);
        Ok(())
    }

    pub fn blob(&mut self, b: &[u8], max: usize) -> Result<(), ProtoError> {
        if b.len() > max {
            return Err(ProtoError::Oversized);
        }
        self.u16(b.len() as u16);
        self.buf.extend_from_slice(b);
        Ok(())
    }
}

pub struct Rd<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Rd<'a> {
    pub fn new(s: &'a [u8]) -> Self {
        Self { s, i: 0 }
    }

    pub fn rest(&self) -> usize {
        self.s.len().saturating_sub(self.i)
    }

    pub fn u8(&mut self) -> Result<u8, ProtoError> {
        if self.i >= self.s.len() {
            return Err(ProtoError::Truncated);
        }
        let v = self.s[self.i];
        self.i += 1;
        Ok(v)
    }

    pub fn exact(&mut self, n: usize) -> Result<&'a [u8], ProtoError> {
        if self.rest() < n {
            return Err(ProtoError::Truncated);
        }
        let out = &self.s[self.i..self.i + n];
        self.i += n;
        Ok(out)
    }

    pub fn arr<const N: usize>(&mut self) -> Result<[u8; N], ProtoError> {
        let s = self.exact(N)?;
        let mut a = [0u8; N];
        a.copy_from_slice(s);
        Ok(a)
    }

    pub fn u16(&mut self) -> Result<u16, ProtoError> {
        Ok(u16::from_be_bytes(self.arr()?))
    }

    pub fn u32(&mut self) -> Result<u32, ProtoError> {
        Ok(u32::from_be_bytes(self.arr()?))
    }

    pub fn u64(&mut self) -> Result<u64, ProtoError> {
        Ok(u64::from_be_bytes(self.arr()?))
    }

    pub fn str(&mut self, max: usize) -> Result<String, ProtoError> {
        let n = self.u16()? as usize;
        if n > max {
            return Err(ProtoError::Oversized);
        }
        let b = self.exact(n)?;
        std::str::from_utf8(b)
            .map(|s| s.to_string())
            .map_err(|_| ProtoError::BadUtf8)
    }

    pub fn blob(&mut self, max: usize) -> Result<Vec<u8>, ProtoError> {
        let n = self.u16()? as usize;
        if n > max {
            return Err(ProtoError::Oversized);
        }
        Ok(self.exact(n)?.to_vec())
    }

    pub fn finish(self) -> Result<(), ProtoError> {
        if self.i != self.s.len() {
            Err(ProtoError::BadLength)
        } else {
            Ok(())
        }
    }
}
