// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use p256::SecretKey;
use rand::rngs::OsRng;
use rand::RngCore;
use serde_json::json;

use crate::crypto::{self, Session};
use crate::error::ProtoError;
use crate::identity::{parse_compressed, Identity, Role};
use crate::msg::*;
use crate::qr::{self, BIND_SECONDS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallPhase {
    Ringing,
    Active,
    Held,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkStatus {
    Idle,
    Linked,
    OutOfRange,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallView {
    pub id: u64,
    pub phase: CallPhase,
    pub incoming: bool,
    pub sim: u8,
    pub number: String,
    pub name: String,
    pub digits: String,
    pub status: Option<u8>,
    pub reason: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmsView {
    pub id: u64,
    pub timestamp_ms: u64,
    pub folder: u8,
    pub sim: u8,
    pub address: String,
    pub body: String,
    pub otp: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryView {
    pub id: u64,
    pub timestamp_ms: u64,
    pub direction: u8,
    pub status: u8,
    pub sim: u8,
    pub number: String,
    pub name: String,
    pub duration_s: u32,
}

#[derive(Clone, Debug)]
struct Bond {
    peer_static: [u8; 33],
    psm: u16,
}

struct Offer {
    token: [u8; 16],
    expiry: u64,
    psm: u16,
    spent: bool,
}

struct Outbound {
    eph: SecretKey,
    nonce_i: [u8; 16],
    payload: Vec<u8>,
    token: [u8; 16],
    peer_static: [u8; 33],
    psm: u16,
    peer_device_id: [u8; 16],
    peer_name: String,
}

struct PendingKeys {
    c2s: [u8; 32],
    s2c: [u8; 32],
    peer_static: [u8; 33],
    transcript: [u8; 32],
    psm: u16,
}

struct Call {
    phase: CallPhase,
    incoming: bool,
    sim: u8,
    number: String,
    name: String,
    digits: String,
    went_active: bool,
    terminal: Option<u8>,
    reason: u8,
}

struct PeerHello {
    tier: u8,
}

#[derive(Debug)]
pub struct Effect {
    pub message: Message,
    pub replies: Vec<Vec<u8>>,
    pub app_error: Option<ProtoError>,
}

pub struct Endpoint {
    identity: Identity,
    role: Role,
    tier: u8,
    bond: Option<Bond>,
    offer: Option<Offer>,
    outbound: Option<Outbound>,
    pending: Option<PendingKeys>,
    session: Option<Session>,
    sent_hello: bool,
    peer_hello: Option<PeerHello>,
    link_up: bool,
    out_of_range: bool,
    failed: bool,
    calls: HashMap<u64, Call>,
    sms: Vec<SmsView>,
    contacts: HashMap<u64, (String, String, String, Vec<u8>)>,
    history: Vec<HistoryView>,
    sims: Vec<SimCard>,
    thread_sim: HashMap<String, u8>,
    next_id: u64,
    now: u64,
    audio: Vec<(u64, u32)>,
    last_error: Option<(u16, String)>,
}

impl Endpoint {
    pub fn new(identity: Identity, role: Role, tier: u8) -> Self {
        Self {
            identity,
            role,
            tier,
            bond: None,
            offer: None,
            outbound: None,
            pending: None,
            session: None,
            sent_hello: false,
            peer_hello: None,
            link_up: false,
            out_of_range: false,
            failed: false,
            calls: HashMap::new(),
            sms: Vec::new(),
            contacts: HashMap::new(),
            history: Vec::new(),
            sims: Vec::new(),
            thread_sim: HashMap::new(),
            next_id: 1,
            now: 0,
            audio: Vec::new(),
            last_error: None,
        }
    }

    pub fn public_key(&self) -> [u8; 33] {
        self.identity.public_compressed()
    }

    pub fn status(&self) -> LinkStatus {
        if self.failed {
            LinkStatus::Failed
        } else if self.out_of_range {
            LinkStatus::OutOfRange
        } else if self.link_up {
            LinkStatus::Linked
        } else {
            LinkStatus::Idle
        }
    }

    pub fn has_session(&self) -> bool {
        self.session.is_some()
    }

    pub fn link_up(&self) -> bool {
        self.link_up
    }

    pub fn traffic_keys(&self) -> Option<([u8; 32], [u8; 32])> {
        self.session.as_ref().map(|s| (s.c2s(), s.s2c()))
    }

    pub fn off_hook(&self) -> bool {
        self.calls
            .values()
            .any(|c| matches!(c.phase, CallPhase::Active | CallPhase::Held))
    }

    pub fn call(&self, id: u64) -> Option<CallView> {
        self.calls.get(&id).map(|c| CallView {
            id,
            phase: c.phase,
            incoming: c.incoming,
            sim: c.sim,
            number: c.number.clone(),
            name: c.name.clone(),
            digits: c.digits.clone(),
            status: c.terminal.map(|_| history_status(c)),
            reason: c.reason,
        })
    }

    pub fn messages(&self) -> &[SmsView] {
        &self.sms
    }

    pub fn contact(&self, id: u64) -> Option<(String, String, String, Vec<u8>)> {
        self.contacts.get(&id).cloned()
    }

    pub fn contact_count(&self) -> usize {
        self.contacts.len()
    }

    pub fn contact_id(&self, number: &str) -> Option<u64> {
        self.contacts.iter().find(|(_, c)| c.1 == number).map(|(id, _)| *id)
    }

    pub fn history(&self) -> &[HistoryView] {
        &self.history
    }

    pub fn sim_for(&self, address: &str) -> Option<u8> {
        self.thread_sim.get(address).copied()
    }

    pub fn audio(&self) -> &[(u64, u32)] {
        &self.audio
    }

    pub fn last_error(&self) -> Option<(u16, String)> {
        self.last_error.clone()
    }

    pub fn sims(&self) -> &[SimCard] {
        &self.sims
    }

    /// Records a public key the caller claims to have seen. This does not make the bag accept them.
    pub fn trust_peer_public(&mut self, peer_static: [u8; 33], psm: u16) {
        self.bond = Some(Bond { peer_static, psm });
    }

    pub fn issue_qr(&mut self, now: u64, psm: u16) -> Result<String, ProtoError> {
        let mut token = [0u8; 16];
        OsRng.fill_bytes(&mut token);
        self.issue_qr_with_token(now, psm, token)
    }

    pub fn issue_qr_with_token(&mut self, now: u64, psm: u16, token: [u8; 16]) -> Result<String, ProtoError> {
        self.require_role(Role::Bag)?;
        self.now = now;
        let expiry = now.saturating_add(BIND_SECONDS);
        let text = qr::encode_qr(&self.identity, expiry, token, psm)?;
        self.offer = Some(Offer { token, expiry, psm, spent: false });
        Ok(text)
    }

    pub fn start_from_qr(&mut self, qr: &str, now: u64) -> Result<Vec<u8>, ProtoError> {
        let eph = SecretKey::random(&mut OsRng);
        let scalar: [u8; 32] = eph.to_bytes().into();
        let mut nonce_i = [0u8; 16];
        OsRng.fill_bytes(&mut nonce_i);
        self.start_from_qr_with(qr, now, scalar, nonce_i)
    }

    pub fn start_from_qr_with(
        &mut self,
        qr_text: &str,
        now: u64,
        eph_scalar: [u8; 32],
        nonce_i: [u8; 16],
    ) -> Result<Vec<u8>, ProtoError> {
        self.require_role(Role::Hand)?;
        self.now = now;
        let parsed = qr::decode_qr(qr_text)?;
        if now > parsed.expiry {
            return Err(ProtoError::ExpiredQr);
        }
        self.outbound_hello(
            Identity::ephemeral(eph_scalar)?,
            nonce_i,
            parsed.token,
            parsed.public_key,
            parsed.psm,
            parsed.device_id,
            parsed.name,
        )
    }

    pub fn start_reconnect(&mut self) -> Result<Vec<u8>, ProtoError> {
        let eph = SecretKey::random(&mut OsRng);
        let scalar: [u8; 32] = eph.to_bytes().into();
        let mut nonce_i = [0u8; 16];
        OsRng.fill_bytes(&mut nonce_i);
        self.start_reconnect_with(scalar, nonce_i)
    }

    pub fn start_reconnect_with(&mut self, eph_scalar: [u8; 32], nonce_i: [u8; 16]) -> Result<Vec<u8>, ProtoError> {
        self.require_role(Role::Hand)?;
        let bond = self.bond.clone().ok_or(ProtoError::NotPaired)?;
        self.outbound_hello(
            Identity::ephemeral(eph_scalar)?,
            nonce_i,
            [0u8; 16],
            bond.peer_static,
            bond.psm,
            [0u8; 16],
            String::new(),
        )
    }

    fn outbound_hello(
        &mut self,
        eph: SecretKey,
        nonce_i: [u8; 16],
        token: [u8; 16],
        peer_static: [u8; 33],
        psm: u16,
        peer_device_id: [u8; 16],
        peer_name: String,
    ) -> Result<Vec<u8>, ProtoError> {
        let eph_pub = compressed_of(&eph)?;
        let mut payload = Vec::with_capacity(82);
        payload.extend_from_slice(&eph_pub);
        payload.extend_from_slice(&self.identity.public_compressed());
        payload.extend_from_slice(&nonce_i);
        let record = crypto::encode_record(1, &payload)?;
        self.outbound = Some(Outbound {
            eph,
            nonce_i,
            payload,
            token,
            peer_static,
            psm,
            peer_device_id,
            peer_name,
        });
        self.pending = None;
        self.session = None;
        self.sent_hello = false;
        self.peer_hello = None;
        self.link_up = false;
        Ok(record)
    }

    pub fn accept_client_hello(&mut self, record: &[u8], now: u64) -> Result<Vec<u8>, ProtoError> {
        let eph = SecretKey::random(&mut OsRng);
        let scalar: [u8; 32] = eph.to_bytes().into();
        let mut nonce_b = [0u8; 16];
        OsRng.fill_bytes(&mut nonce_b);
        self.accept_client_hello_with(record, now, scalar, nonce_b)
    }

    pub fn accept_client_hello_with(
        &mut self,
        record: &[u8],
        now: u64,
        eph_scalar: [u8; 32],
        nonce_b: [u8; 16],
    ) -> Result<Vec<u8>, ProtoError> {
        self.require_role(Role::Bag)?;
        self.now = now;
        let offer = self.offer.as_ref().ok_or(ProtoError::NoOffer)?;
        if now > offer.expiry {
            return Err(ProtoError::ExpiredQr);
        }
        if offer.spent {
            return Err(ProtoError::ReusedToken);
        }
        let token = offer.token;
        let psm = offer.psm;
        self.accept_hello_record(record, token, psm, false, eph_scalar, nonce_b)
    }

    pub fn accept_reconnect(&mut self, record: &[u8], now: u64) -> Result<Vec<u8>, ProtoError> {
        let eph = SecretKey::random(&mut OsRng);
        let scalar: [u8; 32] = eph.to_bytes().into();
        let mut nonce_b = [0u8; 16];
        OsRng.fill_bytes(&mut nonce_b);
        self.accept_reconnect_with(record, now, scalar, nonce_b)
    }

    pub fn accept_reconnect_with(
        &mut self,
        record: &[u8],
        now: u64,
        eph_scalar: [u8; 32],
        nonce_b: [u8; 16],
    ) -> Result<Vec<u8>, ProtoError> {
        self.require_role(Role::Bag)?;
        self.now = now;
        if self.bond.is_none() {
            return Err(ProtoError::NotPaired);
        }
        let psm = self.bond.as_ref().unwrap().psm;
        self.accept_hello_record(record, [0u8; 16], psm, true, eph_scalar, nonce_b)
    }

    fn accept_hello_record(
        &mut self,
        record: &[u8],
        token: [u8; 16],
        psm: u16,
        reconnect: bool,
        eph_scalar: [u8; 32],
        nonce_b: [u8; 16],
    ) -> Result<Vec<u8>, ProtoError> {
        let (kind, payload) = crypto::decode_record(record)?;
        if kind != 1 {
            return Err(ProtoError::UnknownType);
        }
        let hello = crypto::parse_client_hello(&payload)?;
        if reconnect {
            let bond = self.bond.as_ref().ok_or(ProtoError::NotPaired)?;
            if !crypto::ct_eq(&hello.static_key, &bond.peer_static) {
                return Err(ProtoError::NotPaired);
            }
        }
        if let Some(offer) = self.offer.as_mut() {
            if !reconnect {
                offer.spent = true;
            }
        }
        let b_eph = Identity::ephemeral(eph_scalar)?;
        let b_eph_pub = compressed_of(&b_eph)?;
        let keys = derive_bag(
            &b_eph,
            self.identity.secret(),
            &hello.eph,
            &hello.static_key,
            &hello.nonce_i,
            &nonce_b,
            &token,
        )?;
        let hash = crypto::transcript_hash(&payload, &server_payload(&b_eph_pub, &nonce_b), &token);
        self.pending = Some(PendingKeys {
            c2s: keys.0,
            s2c: keys.1,
            peer_static: hello.static_key,
            transcript: hash,
            psm,
        });
        self.session = None;
        self.sent_hello = false;
        self.peer_hello = None;
        self.link_up = false;
        let mut sh = Vec::with_capacity(49);
        sh.extend_from_slice(&b_eph_pub);
        sh.extend_from_slice(&nonce_b);
        crypto::encode_record(2, &sh)
    }

    pub fn accept_server_hello(&mut self, record: &[u8]) -> Result<Vec<u8>, ProtoError> {
        self.require_role(Role::Hand)?;
        let outbound = self.outbound.take().ok_or(ProtoError::SessionDown)?;
        let (kind, payload) = crypto::decode_record(record)?;
        if kind != 2 {
            return Err(ProtoError::UnknownType);
        }
        let (b_eph, nonce_b) = crypto::parse_server_hello(&payload)?;
        let (c2s, s2c) = derive_hand(
            &outbound.eph,
            self.identity.secret(),
            &b_eph,
            &outbound.peer_static,
            &outbound.nonce_i,
            &nonce_b,
            &outbound.token,
        )?;
        let hash = crypto::transcript_hash(&outbound.payload, &payload, &outbound.token);
        let blob = crypto::finished_blob(&c2s, 1, &hash, b"cfin")?;
        self.pending = Some(PendingKeys {
            c2s,
            s2c,
            peer_static: outbound.peer_static,
            transcript: hash,
            psm: outbound.psm,
        });
        let _ = (outbound.peer_device_id, outbound.peer_name);
        crypto::encode_record(3, &blob)
    }

    pub fn accept_client_finished(&mut self, record: &[u8]) -> Result<Vec<u8>, ProtoError> {
        self.require_role(Role::Bag)?;
        let pending = self.pending.as_ref().ok_or(ProtoError::SessionDown)?;
        let (kind, payload) = crypto::decode_record(record)?;
        if kind != 3 {
            return Err(ProtoError::UnknownType);
        }
        if crypto::finished_open(&pending.c2s, 1, &payload, &pending.transcript, b"cfin").is_err() {
            self.pending = None;
            return Err(ProtoError::FinishedMismatch);
        }
        let pending = self.pending.take().unwrap();
        let blob = crypto::finished_blob(&pending.s2c, 2, &pending.transcript, b"sfin")?;
        self.install_session(pending, Role::Bag);
        crypto::encode_record(4, &blob)
    }

    pub fn accept_server_finished(&mut self, record: &[u8]) -> Result<(), ProtoError> {
        self.require_role(Role::Hand)?;
        let pending = self.pending.as_ref().ok_or(ProtoError::SessionDown)?;
        let (kind, payload) = crypto::decode_record(record)?;
        if kind != 4 {
            return Err(ProtoError::UnknownType);
        }
        if crypto::finished_open(&pending.s2c, 2, &payload, &pending.transcript, b"sfin").is_err() {
            self.pending = None;
            return Err(ProtoError::FinishedMismatch);
        }
        let pending = self.pending.take().unwrap();
        self.install_session(pending, Role::Hand);
        Ok(())
    }

    fn install_session(&mut self, pending: PendingKeys, role: Role) {
        self.bond = Some(Bond {
            peer_static: pending.peer_static,
            psm: pending.psm,
        });
        self.session = Some(Session::new(role == Role::Hand, pending.c2s, pending.s2c));
        self.failed = false;
        self.sent_hello = false;
        self.peer_hello = None;
        self.link_up = false;
    }

    pub fn seal_hello(&mut self) -> Result<Vec<u8>, ProtoError> {
        let msg = Message::Hello {
            tier: self.tier,
            proto: 1,
            device_id: self.identity.device_id(),
            name: self.identity.name().to_string(),
        };
        let frame = self.seal_raw(&msg)?;
        self.sent_hello = true;
        self.refresh_link();
        Ok(frame)
    }

    pub fn open_app(&mut self, frame: &[u8]) -> Result<Effect, ProtoError> {
        if self.session.is_none() {
            return Err(ProtoError::SessionDown);
        }
        let opened = self.session.as_mut().unwrap().open(frame);
        let msg = match opened {
            Ok(m) => m,
            Err(e) => {
                self.fail_link();
                return Err(e);
            }
        };
        if self.peer_hello.is_none() {
            match msg {
                Message::Hello { tier, proto, .. } => {
                    if proto != 1 || tier > 1 {
                        self.fail_link();
                        return Err(ProtoError::BadVersion);
                    }
                    self.peer_hello = Some(PeerHello { tier });
                    self.refresh_link();
                    return Ok(Effect { message: msg, replies: vec![], app_error: None });
                }
                _ => {
                    self.fail_link();
                    return Err(ProtoError::HelloRequired);
                }
            }
        }
        if matches!(msg, Message::Hello { .. }) {
            self.fail_link();
            return Err(ProtoError::BadState);
        }
        self.dispatch(msg)
    }

    fn dispatch(&mut self, msg: Message) -> Result<Effect, ProtoError> {
        let mut replies = Vec::new();
        let mut app_error = None;
        match &msg {
            Message::Ping { ms } => {
                let pong = Message::Pong { ms: *ms };
                replies.push(self.seal_raw(&pong)?);
            }
            Message::CallInvite { direction, call_id, sim, number, name } => {
                if let Err(e) = self.apply_invite(*direction, *call_id, *sim, number, name) {
                    app_error = Some(e);
                }
            }
            Message::CallState { call_id, state, reason } => {
                if let Err(e) = self.apply_state(*call_id, *state, *reason) {
                    app_error = Some(e);
                }
            }
            Message::Dtmf { call_id, digit } => {
                if let Err(e) = self.apply_dtmf(*call_id, *digit) {
                    app_error = Some(e);
                }
            }
            Message::Audio { call_id, seq, .. } => {
                if !self.audio_allowed() {
                    app_error = Some(ProtoError::AudioUnavailable);
                    let err = Message::ErrorMsg { code: 6, message: "audio_unavailable".into() };
                    replies.push(self.seal_raw(&err)?);
                } else if let Err(e) = self.apply_audio(*call_id, *seq) {
                    app_error = Some(e);
                }
            }
            Message::Sms { id, timestamp_ms, folder, sim, address, body } => {
                let cursor = self.store_sms(SmsView {
                    id: *id,
                    timestamp_ms: *timestamp_ms,
                    folder: *folder,
                    sim: *sim,
                    address: address.clone(),
                    body: body.clone(),
                    otp: is_otp(body),
                });
                if *sim != SIM_DEFAULT {
                    self.thread_sim.insert(address.clone(), *sim);
                }
                replies.push(self.seal_raw(&Message::SmsAck { id: *id, cursor })?);
            }
            Message::Contact { id, op, name, number, label, photo } => {
                if *op == 0 {
                    self.contacts.insert(*id, (name.clone(), number.clone(), label.clone(), photo.clone()));
                } else if *op == 1 {
                    self.contacts.remove(id);
                } else {
                    app_error = Some(ProtoError::BadState);
                }
                let cursor = *id;
                replies.push(self.seal_raw(&Message::ContactAck { id: *id, cursor })?);
            }
            Message::HistoryPage { entries, .. } => {
                for e in entries {
                    if !self.history.iter().any(|h| h.id == e.id) {
                        self.history.push(HistoryView {
                            id: e.id,
                            timestamp_ms: e.timestamp_ms,
                            direction: e.direction,
                            status: e.status,
                            sim: e.sim,
                            number: e.number.clone(),
                            name: e.name.clone(),
                            duration_s: e.duration_s,
                        });
                    }
                }
            }
            Message::SimList { sims } => self.sims = sims.clone(),
            Message::ErrorMsg { code, message } => {
                self.last_error = Some((*code, message.clone()));
            }
            Message::Pong { .. } | Message::SmsAck { .. } | Message::ContactAck { .. } | Message::SyncCursor { .. } => {}
            Message::Hello { .. } => unreachable!("hello handled above"),
        }
        Ok(Effect { message: msg, replies, app_error })
    }

    pub fn place_call(&mut self, call_id: u64, sim: u8, number: &str, name: &str) -> Result<Vec<u8>, ProtoError> {
        let resolved = self.resolve_sim(number, sim);
        self.apply_invite(DIR_OUTGOING, call_id, resolved, number, name)?;
        self.seal_raw(&Message::CallInvite {
            direction: DIR_OUTGOING,
            call_id,
            sim: resolved,
            number: number.into(),
            name: name.into(),
        })
    }

    pub fn incoming_call(&mut self, call_id: u64, sim: u8, number: &str, name: &str) -> Result<Vec<u8>, ProtoError> {
        let resolved = self.resolve_sim(number, sim);
        self.apply_invite(DIR_INCOMING, call_id, resolved, number, name)?;
        self.seal_raw(&Message::CallInvite {
            direction: DIR_INCOMING,
            call_id,
            sim: resolved,
            number: number.into(),
            name: name.into(),
        })
    }

    pub fn set_call_state(&mut self, call_id: u64, state: u8, reason: u8) -> Result<Vec<u8>, ProtoError> {
        self.apply_state(call_id, state, reason)?;
        self.seal_raw(&Message::CallState { call_id, state, reason })
    }

    pub fn send_dtmf(&mut self, call_id: u64, digit: u8) -> Result<Vec<u8>, ProtoError> {
        self.apply_dtmf(call_id, digit)?;
        self.seal_raw(&Message::Dtmf { call_id, digit })
    }

    pub fn send_audio(&mut self, call_id: u64, seq: u32, opus: &[u8]) -> Result<Vec<u8>, ProtoError> {
        let active = self
            .calls
            .get(&call_id)
            .map(|c| matches!(c.phase, CallPhase::Active | CallPhase::Held))
            .unwrap_or(false);
        if !active {
            return Err(ProtoError::BadState);
        }
        if self.audio_allowed() {
            self.audio.push((call_id, seq));
        }
        self.seal_raw(&Message::Audio { call_id, seq, opus: opus.to_vec() })
    }

    pub fn compose_sms(&mut self, now: u64, address: &str, body: &str, sim: u8) -> Result<Vec<u8>, ProtoError> {
        self.now = now;
        self.enqueue_sms(now, 1, address, body, sim)
    }

    pub fn carrier_sms(&mut self, now: u64, address: &str, body: &str, sim: u8) -> Result<Vec<u8>, ProtoError> {
        self.now = now;
        self.enqueue_sms(now, 0, address, body, sim)
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        match self.role {
            Role::Bag => id,
            Role::Hand => id | (1 << 63),
        }
    }

    fn enqueue_sms(&mut self, now: u64, folder: u8, address: &str, body: &str, sim: u8) -> Result<Vec<u8>, ProtoError> {
        let resolved = self.resolve_sim(address, sim);
        let id = self.alloc_id();
        let view = SmsView {
            id,
            timestamp_ms: now,
            folder,
            sim: resolved,
            address: address.into(),
            body: body.into(),
            otp: is_otp(body),
        };
        let msg = Message::Sms {
            id,
            timestamp_ms: now,
            folder,
            sim: resolved,
            address: address.into(),
            body: body.into(),
        };
        self.store_sms(view);
        self.seal_raw(&msg)
    }

    pub fn resend_sms(&mut self, id: u64) -> Result<Vec<u8>, ProtoError> {
        let sms = self.sms.iter().find(|s| s.id == id).cloned().ok_or(ProtoError::BadState)?;
        self.seal_raw(&Message::Sms {
            id: sms.id,
            timestamp_ms: sms.timestamp_ms,
            folder: sms.folder,
            sim: sms.sim,
            address: sms.address,
            body: sms.body,
        })
    }

    pub fn add_contact(&mut self, name: &str, number: &str, label: &str, photo: &[u8]) -> Result<Vec<u8>, ProtoError> {
        let id = self.alloc_id();
        self.contacts.insert(id, (name.into(), number.into(), label.into(), photo.to_vec()));
        self.seal_raw(&Message::Contact {
            id,
            op: 0,
            name: name.into(),
            number: number.into(),
            label: label.into(),
            photo: photo.to_vec(),
        })
    }

    pub fn replace_contact(&mut self, id: u64, name: &str, number: &str, label: &str, photo: &[u8]) -> Result<Vec<u8>, ProtoError> {
        if !self.contacts.contains_key(&id) {
            return Err(ProtoError::BadState);
        }
        self.contacts.insert(id, (name.into(), number.into(), label.into(), photo.to_vec()));
        self.seal_raw(&Message::Contact {
            id,
            op: 0,
            name: name.into(),
            number: number.into(),
            label: label.into(),
            photo: photo.to_vec(),
        })
    }

    pub fn delete_contact(&mut self, id: u64) -> Result<Vec<u8>, ProtoError> {
        self.contacts.remove(&id);
        self.seal_raw(&Message::Contact {
            id,
            op: 1,
            name: String::new(),
            number: String::new(),
            label: String::new(),
            photo: vec![],
        })
    }

    pub fn send_sims(&mut self, sims: Vec<SimCard>) -> Result<Vec<u8>, ProtoError> {
        self.sims = sims.clone();
        self.seal_raw(&Message::SimList { sims })
    }

    pub fn send_ping(&mut self, ms: u64) -> Result<Vec<u8>, ProtoError> {
        self.seal_raw(&Message::Ping { ms })
    }

    pub fn disconnect(&mut self) {
        self.fail_link();
        self.failed = false;
    }

    fn resolve_sim(&mut self, number: &str, sim: u8) -> u8 {
        let resolved = if sim == SIM_DEFAULT {
            self.thread_sim.get(number).copied().unwrap_or(0)
        } else {
            sim
        };
        self.thread_sim.insert(number.to_string(), resolved);
        resolved
    }

    fn apply_invite(&mut self, direction: u8, call_id: u64, sim: u8, number: &str, name: &str) -> Result<(), ProtoError> {
        if direction > 1 || self.calls.contains_key(&call_id) {
            return Err(ProtoError::BadState);
        }
        let sim = if sim == SIM_DEFAULT {
            self.thread_sim.get(number).copied().unwrap_or(0)
        } else {
            sim
        };
        self.thread_sim.insert(number.to_string(), sim);
        self.calls.insert(
            call_id,
            Call {
                phase: CallPhase::Ringing,
                incoming: direction == DIR_INCOMING,
                sim,
                number: number.into(),
                name: name.into(),
                digits: String::new(),
                went_active: false,
                terminal: None,
                reason: 0,
            },
        );
        Ok(())
    }

    fn apply_state(&mut self, call_id: u64, state: u8, reason: u8) -> Result<(), ProtoError> {
        let call = self.calls.get_mut(&call_id).ok_or(ProtoError::BadState)?;
        let allowed = match (call.phase, state) {
            (CallPhase::Ringing, ST_ACTIVE) => {
                call.phase = CallPhase::Active;
                call.went_active = true;
                true
            }
            (CallPhase::Held, ST_ACTIVE) => {
                call.phase = CallPhase::Active;
                true
            }
            (CallPhase::Active, ST_HELD) => {
                call.phase = CallPhase::Held;
                true
            }
            (CallPhase::Ringing, ST_DECLINED | ST_BUSY | ST_FAILED | ST_ENDED) => {
                call.phase = CallPhase::Ended;
                true
            }
            (CallPhase::Active | CallPhase::Held, ST_ENDED | ST_FAILED) => {
                call.phase = CallPhase::Ended;
                true
            }
            _ => false,
        };
        if !allowed {
            return Err(ProtoError::BadState);
        }
        if self.calls.get(&call_id).map(|c| c.phase) == Some(CallPhase::Ended) {
            if let Some(c) = self.calls.get_mut(&call_id) {
                c.terminal = Some(state);
                c.reason = reason;
            }
            self.push_history(call_id);
        }
        Ok(())
    }

    fn apply_dtmf(&mut self, call_id: u64, digit: u8) -> Result<(), ProtoError> {
        if !valid_dtmf(digit) {
            return Err(ProtoError::BadDigit);
        }
        let call = self.calls.get_mut(&call_id).ok_or(ProtoError::BadState)?;
        if !matches!(call.phase, CallPhase::Active | CallPhase::Held) {
            return Err(ProtoError::BadState);
        }
        call.digits.push(digit as char);
        Ok(())
    }

    fn apply_audio(&mut self, call_id: u64, seq: u32) -> Result<(), ProtoError> {
        let call = self.calls.get(&call_id).ok_or(ProtoError::BadState)?;
        if !matches!(call.phase, CallPhase::Active | CallPhase::Held) {
            return Err(ProtoError::BadState);
        }
        self.audio.push((call_id, seq));
        Ok(())
    }

    fn push_history(&mut self, call_id: u64) {
        if self.history.iter().any(|h| h.id == call_id) {
            return;
        }
        let Some(call) = self.calls.get(&call_id) else { return };
        self.history.push(HistoryView {
            id: call_id,
            timestamp_ms: self.now,
            direction: if call.incoming { DIR_INCOMING } else { DIR_OUTGOING },
            status: history_status(call),
            sim: call.sim,
            number: call.number.clone(),
            name: call.name.clone(),
            duration_s: 0,
        });
    }

    fn store_sms(&mut self, sms: SmsView) -> u64 {
        if !self.sms.iter().any(|s| s.id == sms.id) {
            self.sms.push(sms);
        }
        self.sms.iter().map(|s| s.id).max().unwrap_or(0)
    }

    fn audio_allowed(&self) -> bool {
        self.tier == 1 && self.peer_hello.as_ref().map(|p| p.tier) == Some(1)
    }

    fn seal_raw(&mut self, msg: &Message) -> Result<Vec<u8>, ProtoError> {
        if !matches!(msg, Message::Hello { .. }) && !self.sent_hello {
            return Err(ProtoError::HelloRequired);
        }
        let session = self.session.as_mut().ok_or(ProtoError::SessionDown)?;
        session.seal(msg, 0)
    }

    fn refresh_link(&mut self) {
        self.link_up = self.session.is_some() && self.sent_hello && self.peer_hello.is_some();
        if self.link_up {
            self.out_of_range = false;
            self.failed = false;
        }
    }

    fn fail_link(&mut self) {
        self.session = None;
        self.pending = None;
        self.outbound = None;
        self.sent_hello = false;
        self.peer_hello = None;
        self.link_up = false;
        self.out_of_range = true;
        self.failed = true;
        let ids: Vec<u64> = self
            .calls
            .iter()
            .filter(|(_, c)| c.phase != CallPhase::Ended)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            let _ = self.apply_state(id, ST_ENDED, 7);
        }
    }

    fn require_role(&self, role: Role) -> Result<(), ProtoError> {
        if self.role != role {
            Err(ProtoError::WrongRole)
        } else {
            Ok(())
        }
    }
}

fn history_status(call: &Call) -> u8 {
    match call.terminal {
        Some(ST_DECLINED) => 2,
        Some(ST_BUSY) => 3,
        Some(ST_FAILED) => 4,
        Some(ST_ENDED) if call.went_active && call.incoming => 0,
        Some(ST_ENDED) if call.went_active && !call.incoming => 5,
        Some(ST_ENDED) if call.incoming => 1,
        _ => 4,
    }
}

fn compressed_of(secret: &SecretKey) -> Result<[u8; 33], ProtoError> {
    use p256::elliptic_curve::sec1::ToEncodedPoint;
    let point = secret.public_key().to_encoded_point(true);
    let b = point.as_bytes();
    if b.len() != 33 {
        return Err(ProtoError::BadPublicKey);
    }
    let mut out = [0u8; 33];
    out.copy_from_slice(b);
    Ok(out)
}

fn server_payload(eph: &[u8; 33], nonce_b: &[u8; 16]) -> Vec<u8> {
    let mut sh = Vec::with_capacity(49);
    sh.extend_from_slice(eph);
    sh.extend_from_slice(nonce_b);
    sh
}

fn derive_bag(
    b_eph: &SecretKey,
    b_static: &SecretKey,
    i_eph: &[u8; 33],
    i_static: &[u8; 33],
    nonce_i: &[u8; 16],
    nonce_b: &[u8; 16],
    token: &[u8; 16],
) -> Result<([u8; 32], [u8; 32]), ProtoError> {
    let i_eph = parse_compressed(i_eph)?;
    let i_static = parse_compressed(i_static)?;
    let dh1 = crypto::ecdh(b_eph, &i_eph);
    let dh2 = crypto::ecdh(b_static, &i_eph);
    let dh3 = crypto::ecdh(b_eph, &i_static);
    let dh4 = crypto::ecdh(b_static, &i_static);
    Ok(crypto::derive_keys(&dh1, &dh2, &dh3, &dh4, nonce_i, nonce_b, token))
}

fn derive_hand(
    i_eph: &SecretKey,
    i_static: &SecretKey,
    b_eph: &[u8; 33],
    b_static: &[u8; 33],
    nonce_i: &[u8; 16],
    nonce_b: &[u8; 16],
    token: &[u8; 16],
) -> Result<([u8; 32], [u8; 32]), ProtoError> {
    let b_eph = parse_compressed(b_eph)?;
    let b_static = parse_compressed(b_static)?;
    let dh1 = crypto::ecdh(i_eph, &b_eph);
    let dh2 = crypto::ecdh(i_eph, &b_static);
    let dh3 = crypto::ecdh(i_static, &b_eph);
    let dh4 = crypto::ecdh(i_static, &b_static);
    Ok(crypto::derive_keys(&dh1, &dh2, &dh3, &dh4, nonce_i, nonce_b, token))
}

pub fn pair(bag: &mut Endpoint, hand: &mut Endpoint, now: u64, psm: u16) -> Result<(), ProtoError> {
    let qr = bag.issue_qr(now, psm)?;
    let ch = hand.start_from_qr(&qr, now)?;
    let sh = bag.accept_client_hello(&ch, now)?;
    let cf = hand.accept_server_hello(&sh)?;
    let sf = bag.accept_client_finished(&cf)?;
    hand.accept_server_finished(&sf)?;
    let hello_b = bag.seal_hello()?;
    let hello_h = hand.seal_hello()?;
    bag.open_app(&hello_h)?;
    hand.open_app(&hello_b)?;
    Ok(())
}

pub fn reconnect(bag: &mut Endpoint, hand: &mut Endpoint, now: u64) -> Result<(), ProtoError> {
    let ch = hand.start_reconnect()?;
    let sh = bag.accept_reconnect(&ch, now)?;
    let cf = hand.accept_server_hello(&sh)?;
    let sf = bag.accept_client_finished(&cf)?;
    hand.accept_server_finished(&sf)?;
    let hello_b = bag.seal_hello()?;
    let hello_h = hand.seal_hello()?;
    bag.open_app(&hello_h)?;
    hand.open_app(&hello_b)?;
    Ok(())
}

/// Deliver `frame` and every automatic reply until the pipe is quiet.
pub fn pump(from: &mut Endpoint, to: &mut Endpoint, frame: Vec<u8>) -> Result<(), ProtoError> {
    let mut queue = vec![(false, frame)];
    let mut guard = 0;
    while let Some((from_sender, bytes)) = queue.pop() {
        guard += 1;
        if guard > 32 {
            return Err(ProtoError::BadState);
        }
        let effect = if from_sender { from.open_app(&bytes)? } else { to.open_app(&bytes)? };
        for reply in effect.replies {
            queue.push((!from_sender, reply));
        }
    }
    Ok(())
}

pub fn reference_fixture_json() -> String {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(0x48414E44);
    let bag_id = Identity::generate_with(&mut rng, "Bag").unwrap();
    let hand_id = Identity::generate_with(&mut rng, "Hand").unwrap();
    let bag_scalar = bag_id.scalar();
    let hand_scalar = hand_id.scalar();
    let bag_device = bag_id.device_id();
    let hand_device = hand_id.device_id();
    let eph_h = SecretKey::random(&mut rng);
    let eph_b = SecretKey::random(&mut rng);
    let eph_h_scalar: [u8; 32] = eph_h.to_bytes().into();
    let eph_b_scalar: [u8; 32] = eph_b.to_bytes().into();
    let mut nonce_i = [0u8; 16];
    let mut nonce_b = [0u8; 16];
    let mut token = [0u8; 16];
    rng.fill_bytes(&mut nonce_i);
    rng.fill_bytes(&mut nonce_b);
    rng.fill_bytes(&mut token);

    let mut bag = Endpoint::new(bag_id, Role::Bag, 0);
    let mut hand = Endpoint::new(hand_id, Role::Hand, 1);
    let now = 1_700_000_000u64;
    let qr = bag.issue_qr_with_token(now, 79, token).unwrap();
    let ch = hand.start_from_qr_with(&qr, now, eph_h_scalar, nonce_i).unwrap();
    let sh = bag.accept_client_hello_with(&ch, now, eph_b_scalar, nonce_b).unwrap();
    let cf = hand.accept_server_hello(&sh).unwrap();
    let sf = bag.accept_client_finished(&cf).unwrap();
    hand.accept_server_finished(&sf).unwrap();
    let hello_h = hand.seal_hello().unwrap();
    let hello_b = bag.seal_hello().unwrap();
    bag.open_app(&hello_h).unwrap();
    hand.open_app(&hello_b).unwrap();
    let ping = hand.send_ping(42).unwrap();
    let keys = hand.traffic_keys().unwrap();

    let mut tampered = ping.clone();
    let last = tampered.len() - 1;
    tampered[last] ^= 0x01;

    json!({
        "version": 1,
        "now": now,
        "psm": 79,
        "token_hex": hex(&token),
        "nonce_i_hex": hex(&nonce_i),
        "nonce_b_hex": hex(&nonce_b),
        "bag_scalar_hex": hex(&bag_scalar),
        "hand_scalar_hex": hex(&hand_scalar),
        "bag_eph_scalar_hex": hex(&eph_b_scalar),
        "hand_eph_scalar_hex": hex(&eph_h_scalar),
        "bag_device_id_hex": hex(&bag_device),
        "hand_device_id_hex": hex(&hand_device),
        "bag_name": "Bag",
        "hand_name": "Hand",
        "qr": qr,
        "client_hello_hex": hex(&ch),
        "server_hello_hex": hex(&sh),
        "client_finished_hex": hex(&cf),
        "server_finished_hex": hex(&sf),
        "c2s_key_hex": hex(&keys.0),
        "s2c_key_hex": hex(&keys.1),
        "hello_hand_hex": hex(&hello_h),
        "hello_bag_hex": hex(&hello_b),
        "ping_hand_hex": hex(&ping),
        "tampered_ping_hex": hex(&tampered),
    })
    .to_string()
}

fn hex(data: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        s.push(H[(b >> 4) as usize] as char);
        s.push(H[(b & 0x0f) as usize] as char);
    }
    s
}
