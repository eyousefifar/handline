// SPDX-License-Identifier: Apache-2.0

use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::{PublicKey, SecretKey};
use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};

use crate::error::ProtoError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Bag,
    Hand,
}

pub struct Identity {
    device_id: [u8; 16],
    name: String,
    scalar: [u8; 32],
    secret: SecretKey,
    public_compressed: [u8; 33],
}

impl Identity {
    pub fn generate(name: &str) -> Result<Self, ProtoError> {
        let mut rng = OsRng;
        Self::generate_with(&mut rng, name)
    }

    pub fn generate_with(rng: &mut (impl RngCore + CryptoRng), name: &str) -> Result<Self, ProtoError> {
        let secret = SecretKey::random(&mut *rng);
        let mut device_id = [0u8; 16];
        rng.fill_bytes(&mut device_id);
        let scalar: [u8; 32] = secret.to_bytes().into();
        Self::from_parts(device_id, name, scalar)
    }

    pub fn from_parts(device_id: [u8; 16], name: &str, scalar: [u8; 32]) -> Result<Self, ProtoError> {
        if name.len() > 64 {
            return Err(ProtoError::Oversized);
        }
        let secret = SecretKey::from_slice(&scalar).map_err(|_| ProtoError::Crypto)?;
        let public = secret.public_key();
        let compressed = public.to_encoded_point(true);
        let bytes = compressed.as_bytes();
        if bytes.len() != 33 {
            return Err(ProtoError::BadPublicKey);
        }
        let mut public_compressed = [0u8; 33];
        public_compressed.copy_from_slice(bytes);
        Ok(Self {
            device_id,
            name: name.to_string(),
            scalar,
            secret,
            public_compressed,
        })
    }

    pub fn device_id(&self) -> [u8; 16] {
        self.device_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn scalar(&self) -> [u8; 32] {
        self.scalar
    }

    pub fn public_compressed(&self) -> [u8; 33] {
        self.public_compressed
    }

    pub(crate) fn secret(&self) -> &SecretKey {
        &self.secret
    }

    pub(crate) fn ephemeral(scalar: [u8; 32]) -> Result<SecretKey, ProtoError> {
        SecretKey::from_slice(&scalar).map_err(|_| ProtoError::Crypto)
    }
}

pub fn parse_compressed(bytes: &[u8]) -> Result<PublicKey, ProtoError> {
    PublicKey::from_sec1_bytes(bytes).map_err(|_| ProtoError::BadPublicKey)
}
