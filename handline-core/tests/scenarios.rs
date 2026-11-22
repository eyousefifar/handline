// SPDX-License-Identifier: Apache-2.0

use handline_core::*;

fn ends(tier_bag: u8, tier_hand: u8) -> (Endpoint, Endpoint) {
    let bag = Endpoint::new(Identity::generate("Bag").unwrap(), Role::Bag, tier_bag);
    let hand = Endpoint::new(Identity::generate("Hand").unwrap(), Role::Hand, tier_hand);
    (bag, hand)
}

fn linked() -> (Endpoint, Endpoint) {
    let (mut bag, mut hand) = ends(1, 1);
    pair(&mut bag, &mut hand, 1_700_000_000, 79).unwrap();
    assert_eq!(bag.status(), LinkStatus::Linked);
    assert_eq!(hand.status(), LinkStatus::Linked);
    (bag, hand)
}

#[test]
fn incoming_answer_dtmf_hangup() {
    let (mut bag, mut hand) = linked();
    let invite = bag.incoming_call(7, 0, "+15551212", "Sara").unwrap();
    pump(&mut bag, &mut hand, invite).unwrap();
    assert_eq!(hand.call(7).unwrap().phase, CallPhase::Ringing);
    assert!(hand.call(7).unwrap().incoming);
    let answer = hand.set_call_state(7, 1, 0).unwrap();
    pump(&mut hand, &mut bag, answer).unwrap();
    assert!(bag.off_hook() && hand.off_hook());
    let tone = hand.send_dtmf(7, b'5').unwrap();
    pump(&mut hand, &mut bag, tone).unwrap();
    assert_eq!(bag.call(7).unwrap().digits, "5");
    assert_eq!(hand.call(7).unwrap().digits, "5");
    let bye = bag.set_call_state(7, 3, 0).unwrap();
    pump(&mut bag, &mut hand, bye).unwrap();
    assert!(!bag.off_hook() && !hand.off_hook());
    assert_eq!(bag.call(7).unwrap().phase, CallPhase::Ended);
    assert_eq!(hand.call(7).unwrap().status, Some(0));
}

#[test]
fn decline_leaves_the_bag_on_hook() {
    let (mut bag, mut hand) = linked();
    let invite = bag.incoming_call(3, 1, "+100", "Ada").unwrap();
    pump(&mut bag, &mut hand, invite).unwrap();
    let declined = hand.set_call_state(3, 4, 0).unwrap();
    pump(&mut hand, &mut bag, declined).unwrap();
    assert!(!bag.off_hook() && !hand.off_hook());
    assert_eq!(bag.call(3).unwrap().status, Some(2));
    assert_eq!(hand.history()[0].status, 2);
}

#[test]
fn missed_when_the_caller_hangs_up_first() {
    let (mut bag, mut hand) = linked();
    let invite = bag.incoming_call(4, 0, "+100", "Bea").unwrap();
    pump(&mut bag, &mut hand, invite).unwrap();
    let ended = bag.set_call_state(4, 3, 0).unwrap();
    pump(&mut bag, &mut hand, ended).unwrap();
    assert_eq!(hand.call(4).unwrap().status, Some(1));
}

#[test]
fn sim_choice_sticks_to_the_number() {
    let (mut bag, mut hand) = linked();
    let c1 = hand.place_call(11, 0, "+200", "Sam").unwrap();
    pump(&mut hand, &mut bag, c1).unwrap();
    assert_eq!(bag.call(11).unwrap().sim, 0);
    let c2 = hand.place_call(12, 0xFF, "+200", "Sam").unwrap();
    pump(&mut hand, &mut bag, c2).unwrap();
    assert_eq!(bag.call(12).unwrap().sim, 0);
    let c3 = hand.place_call(13, 1, "+200", "Sam").unwrap();
    pump(&mut hand, &mut bag, c3).unwrap();
    let c4 = hand.place_call(14, 0xFF, "+200", "Sam").unwrap();
    pump(&mut hand, &mut bag, c4).unwrap();
    assert_eq!(hand.call(14).unwrap().sim, 1);
    assert_eq!(bag.call(14).unwrap().sim, 1);
    assert_eq!(hand.sim_for("+200"), Some(1));
}

#[test]
fn sms_sync_does_not_duplicate_and_flags_a_code() {
    let (mut bag, mut hand) = linked();
    let first = bag.carrier_sms(10, "+300", "123456", 0).unwrap();
    let id = bag.messages()[0].id;
    pump(&mut bag, &mut hand, first).unwrap();
    assert_eq!(hand.messages().len(), 1);
    assert!(hand.messages()[0].otp);
    let lunch = bag.carrier_sms(11, "+300", "Lunch at 1234 Main", 0).unwrap();
    pump(&mut bag, &mut hand, lunch).unwrap();
    assert!(!hand.messages()[1].otp);
    let reply = hand.compose_sms(12, "+300", "Your code is 482193", 0xFF).unwrap();
    pump(&mut hand, &mut bag, reply).unwrap();
    assert!(bag.messages().iter().any(|m| m.otp && m.body.contains("482193")));
    let before = hand.messages().len();
    bag.disconnect();
    hand.disconnect();
    assert_eq!(hand.status(), LinkStatus::OutOfRange);
    reconnect(&mut bag, &mut hand, 1_700_000_100).unwrap();
    let again = bag.resend_sms(id).unwrap();
    pump(&mut bag, &mut hand, again).unwrap();
    assert_eq!(hand.messages().len(), before);
}

#[test]
fn contacts_roundtrip_including_photo_and_delete() {
    let (mut bag, mut hand) = linked();
    let frame = hand.add_contact("Neda", "+400", "home", b"pic-1").unwrap();
    pump(&mut hand, &mut bag, frame).unwrap();
    let cid = hand.contact_id("+400").unwrap();
    assert_eq!(bag.contact(cid).unwrap().0, "Neda");
    let photo = hand.replace_contact(cid, "Neda", "+400", "home", b"pic-2").unwrap();
    pump(&mut hand, &mut bag, photo).unwrap();
    assert_eq!(bag.contact(cid).unwrap().3, b"pic-2");
    let gone = hand.delete_contact(cid).unwrap();
    pump(&mut hand, &mut bag, gone).unwrap();
    assert_eq!(bag.contact_count(), 0);
    assert_eq!(hand.contact_count(), 0);
}

#[test]
fn expired_and_reused_qr_are_refused() {
    let (mut bag, mut hand) = ends(0, 0);
    let qr = bag.issue_qr(1_000, 79).unwrap();
    assert_eq!(hand.start_from_qr(&qr, 1_301).unwrap_err(), ProtoError::ExpiredQr);
    let ch = hand.start_from_qr(&qr, 1_000).unwrap();
    assert_eq!(bag.accept_client_hello(&ch, 1_301).unwrap_err(), ProtoError::ExpiredQr);
    let sh = bag.accept_client_hello(&ch, 1_000).unwrap();
    let cf = hand.accept_server_hello(&sh).unwrap();
    let sf = bag.accept_client_finished(&cf).unwrap();
    hand.accept_server_finished(&sf).unwrap();
    let mut other = Endpoint::new(Identity::generate("Other").unwrap(), Role::Hand, 0);
    let again = other.start_from_qr(&qr, 1_000).unwrap();
    assert_eq!(bag.accept_client_hello(&again, 1_000).unwrap_err(), ProtoError::ReusedToken);
}

#[test]
fn mutated_transcript_does_not_install_keys() {
    let (mut bag, mut hand) = ends(0, 0);
    let qr = bag.issue_qr(5_000, 79).unwrap();
    let mut ch = hand.start_from_qr(&qr, 5_000).unwrap();
    ch[70] ^= 0x01;
    let sh = bag.accept_client_hello(&ch, 5_000).unwrap();
    let cf = hand.accept_server_hello(&sh).unwrap();
    assert_eq!(bag.accept_client_finished(&cf).unwrap_err(), ProtoError::FinishedMismatch);
    assert!(!bag.has_session());
    assert!(!hand.has_session());
}

#[test]
fn a_third_phone_cannot_join_without_a_new_qr() {
    let (mut bag, hand) = linked();
    assert!(bag.has_session());
    let mut spy = Endpoint::new(Identity::generate("Spy").unwrap(), Role::Hand, 0);
    spy.trust_peer_public(bag.public_key(), 79);
    let ch = spy.start_reconnect().unwrap();
    assert_eq!(bag.accept_reconnect(&ch, 1_700_000_000).unwrap_err(), ProtoError::NotPaired);
    assert!(bag.has_session());
    assert_eq!(hand.status(), LinkStatus::Linked);
}

#[test]
fn replay_kills_the_session_and_the_call() {
    let (mut bag, mut hand) = linked();
    let invite = bag.incoming_call(9, 0, "+1", "A").unwrap();
    pump(&mut bag, &mut hand, invite).unwrap();
    let answer = hand.set_call_state(9, 1, 0).unwrap();
    pump(&mut hand, &mut bag, answer).unwrap();
    let frame = hand.send_ping(1).unwrap();
    bag.open_app(&frame).unwrap();
    assert_eq!(bag.open_app(&frame).unwrap_err(), ProtoError::Replay);
    assert_eq!(bag.status(), LinkStatus::Failed);
    assert!(!bag.off_hook());
    assert!(!bag.has_session());
    assert_eq!(bag.call(9).unwrap().phase, CallPhase::Ended);
}

#[test]
fn counter_far_ahead_is_rejected_and_a_small_gap_is_not() {
    let (mut bag, mut hand) = linked();
    let mut frames = Vec::new();
    for i in 0..33 {
        frames.push(hand.send_ping(i).unwrap());
    }
    assert_eq!(bag.open_app(&frames[32]).unwrap_err(), ProtoError::CounterAhead);
    assert_eq!(bag.status(), LinkStatus::Failed);

    let (mut bag, mut hand) = linked();
    let mut frames = Vec::new();
    for i in 0..32 {
        frames.push(hand.send_ping(i).unwrap());
    }
    bag.open_app(&frames[31]).unwrap();
    bag.open_app(&frames[0]).unwrap();
    assert!(bag.has_session());
}

#[test]
fn out_of_range_ends_audio_and_reconnect_does_not_resume_the_call() {
    let (mut bag, mut hand) = linked();
    let invite = bag.incoming_call(21, 0, "+9", "Q").unwrap();
    pump(&mut bag, &mut hand, invite).unwrap();
    let answer = hand.set_call_state(21, 1, 0).unwrap();
    pump(&mut hand, &mut bag, answer).unwrap();
    let audio = bag.send_audio(21, 0, &[1, 2, 3, 4]).unwrap();
    pump(&mut bag, &mut hand, audio).unwrap();
    assert_eq!(hand.audio().len(), 1);
    bag.disconnect();
    hand.disconnect();
    assert_eq!(hand.status(), LinkStatus::OutOfRange);
    assert!(!hand.off_hook());
    assert_eq!(hand.call(21).unwrap().phase, CallPhase::Ended);
    reconnect(&mut bag, &mut hand, 1_700_000_050).unwrap();
    assert_eq!(hand.send_audio(21, 1, &[9]).unwrap_err(), ProtoError::BadState);
    assert_eq!(hand.audio().len(), 1);
    assert_eq!(hand.status(), LinkStatus::Linked);
}

#[test]
fn tier_a_does_not_play_audio() {
    let (mut bag, mut hand) = ends(1, 0);
    pair(&mut bag, &mut hand, 50, 79).unwrap();
    let invite = bag.incoming_call(2, 0, "+1", "Z").unwrap();
    pump(&mut bag, &mut hand, invite).unwrap();
    let answer = hand.set_call_state(2, 1, 0).unwrap();
    pump(&mut hand, &mut bag, answer).unwrap();
    let frame = bag.send_audio(2, 0, &[7, 7]).unwrap();
    let effect = hand.open_app(&frame).unwrap();
    assert_eq!(effect.app_error, Some(ProtoError::AudioUnavailable));
    assert!(hand.audio().is_empty());
    let err = effect.replies[0].clone();
    pump(&mut hand, &mut bag, err).unwrap();
    assert_eq!(bag.last_error().unwrap().0, 6);
}

#[test]
fn both_tiers_b_play_audio() {
    let (mut bag, mut hand) = linked();
    let invite = bag.incoming_call(2, 0, "+1", "Z").unwrap();
    pump(&mut bag, &mut hand, invite).unwrap();
    let answer = hand.set_call_state(2, 1, 0).unwrap();
    pump(&mut hand, &mut bag, answer).unwrap();
    let audio = bag.send_audio(2, 4, &[7, 7]).unwrap();
    pump(&mut bag, &mut hand, audio).unwrap();
    assert_eq!(hand.audio(), &[(2, 4)]);
}

#[test]
fn otp_detector() {
    assert!(is_otp("123456"));
    assert!(is_otp("  482193 "));
    assert!(is_otp("Your code is 482193"));
    assert!(is_otp("verification 998877"));
    assert!(!is_otp("Lunch at 1234 Main"));
    assert!(!is_otp("otp"));
    assert!(!is_otp("hello"));
}

#[test]
fn message_bodies_roundtrip() {
    let msg = Message::Sms {
        id: 3,
        timestamp_ms: 9,
        folder: 0,
        sim: 1,
        address: "+1".into(),
        body: "hi".into(),
    };
    let bytes = msg.encode().unwrap();
    assert_eq!(decode_body_for_test(msg.type_byte(), &bytes).unwrap(), msg);
}

#[test]
fn fixture_file_matches_the_reference() {
    let got = reference_fixture_json();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/reference.json");
    if std::env::var("HANDLINE_EXPORT").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &got).unwrap();
    }
    let file = std::fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(file, got);
}
