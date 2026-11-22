# Threat model

The person using Handline owns both phones. The attacker is near the Bluetooth radios, or has picked up an unlocked phone, or has photographed a QR code.

## Nearby eavesdropper

They see handshake public keys and GCM ciphertext. They do not see the token or the identity scalars. A flipped ciphertext bit fails the tag and tears down the session. The test `mutated_transcript_does_not_install_keys` and the tampered-ping vector cover this.

## Replay

A resent frame is rejected and the session is destroyed. Live calls end. The bond remains so a real reconnect can proceed. Covered by `replay_kills_the_session_and_the_call`.

## Third phone

Without the current QR, a new static key cannot join. A reconnect with a different static key is rejected and the existing session stays up. Covered by `a_third_phone_cannot_join_without_a_new_qr`.

## Stolen QR photo

Within five minutes, and before the real phone finishes, the photo is enough to start a handshake. After five minutes the code is dead. After one success it is dead. This is accepted. Pairing is in person.

## Stolen iPhone

The bond is on the device. The owner unpairs from the bag phone, which drops the bond the next time that phone is told to forget the peer. Messages were never on a server.

## Helper abuse

The helper installs only after a code the owner typed, from a window on the same phone. A probe that cannot open the voice-call source does not record a call. The state machine tests reject a short code, an install before pairing, and an uplink-only probe.

## What we do not claim

The Bluetooth controller, the operating system, and a person who can unlock either phone are outside this model. A compromised phone can read what the app can read. Handline does not add a server that would make that worse.
