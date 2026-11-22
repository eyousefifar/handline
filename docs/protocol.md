# Protocol

Handline speaks one original protocol. `handline-core` is the reference. Swift and Kotlin must reproduce `handline-core/testdata/reference.json` byte for byte. Integers are big-endian. Strings are UTF-8 with an explicit length. A parser rejects truncated input, trailing bytes, unknown versions, and oversize fields. It does not panic.

## Transport

One Bluetooth LE L2CAP connection-oriented channel. The bag phone picks the PSM and puts it in the QR code. Each L2CAP SDU is exactly one handshake record or exactly one application frame. There is no second stream inside an SDU.

The link is ordered. Receivers still keep a 32-counter window so a small reorder does not kill a call. A counter more than 32 ahead, or a counter already seen, destroys the session.

## QR code

Text form: `handline:` plus base64url of the binary payload, no padding.

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 4 | magic `HL01` |
| 4 | 1 | version `1` |
| 5 | 16 | bag device id |
| 21 | 33 | bag identity public key, SEC1 compressed |
| 54 | 8 | expiry, unix seconds |
| 62 | 16 | one-time token |
| 78 | 2 | L2CAP PSM |
| 80 | 1 | name length, at most 64 |
| 81 | n | UTF-8 name |

The bag phone sets expiry to now + 300 seconds. A late hello does not burn the token. A hello accepted once burns it. Issuing a new code replaces the previous offer. While a code is unspent, whoever can see it can start pairing. That is the in-person assumption. After pairing, a reconnect is accepted only from the bonded static key.

## Handshake records

`type:u8 || length:u16 || payload`. The length is the payload only. The buffer must end at the payload.

| Type | Name | Payload |
| --- | --- | --- |
| 1 | ClientHello | ephemeral pub 33, static pub 33, nonce 16 |
| 2 | ServerHello | ephemeral pub 33, nonce 16 |
| 3 | ClientFinished | 48-byte AES-GCM blob |
| 4 | ServerFinished | 48-byte AES-GCM blob |

The token is not on the wire. Both sides already know it: from the QR on first pair, or 16 zero bytes on reconnect.

Key schedule, in this order, each value the 32-byte X coordinate of P-256 ECDH:

1. hand ephemeral with bag ephemeral
2. hand ephemeral with bag static
3. hand static with bag ephemeral
4. hand static with bag static

`HKDF-SHA256` salt is `nonce_i || nonce_b || token`. Info is the ASCII string `handline v1`. The 64-byte output splits into the hand-to-bag key and then the bag-to-hand key.

The transcript hash is SHA-256 over `client_hello_payload || server_hello_payload || token`. ClientFinished seals that hash under the hand-to-bag key, direction byte 1, counter 0, additional data `cfin`. ServerFinished uses the bag-to-hand key, direction 2, counter 0, additional data `sfin`. Either failure deletes the pending keys and does not install a session. A failed reconnect does not delete an existing bond.

## Frames

After both Finished messages, application data uses this frame. Header is 9 bytes.

| Offset | Size | Field |
| --- | --- | --- |
| 0 | 1 | version `1` |
| 1 | 1 | message type |
| 2 | 1 | flags, only bit 0 is defined |
| 3 | 4 | counter |
| 7 | 2 | ciphertext length, not including the tag |
| 9 | n | ciphertext |
| 9+n | 16 | GCM tag |

Additional data is version, type, flags, and the counter: 7 bytes. The length field is not in the additional data. Plaintext is at most 4096 bytes. Counters start at 1 in each direction. Counter 0 is reserved for Finished.

The 12-byte nonce is `direction || 7 zero bytes || counter`. Direction 1 is hand to bag. Direction 2 is bag to hand.

Decrypt first. A bad tag destroys the session. An authenticated counter outside the window, or a repeated counter, destroys the session. Authenticated flags other than bit 0 destroy the session. The first frame in each direction must be Hello. A second Hello destroys the session. An unknown message type destroys the session.

A bad call transition, a bad DTMF digit, or a refusal of audio does not destroy the session.

## Messages

The plaintext is the body. The type lives in the frame.

| Type | Body |
| --- | --- |
| 1 Hello | tier u8, proto u8, device id 16, name (u16) |
| 2 Ping | unix ms u64 |
| 3 Pong | unix ms u64 |
| 4 CallInvite | direction u8, call id u64, sim u8, number, name |
| 5 CallState | call id u64, state u8, reason u8 |
| 6 Dtmf | call id u64, digit u8 |
| 7 Audio | call id u64, seq u32, opus bytes (u16, max 512) |
| 8 Sms | id u64, time u64, folder u8, sim u8, address, body (max 3500) |
| 9 SmsAck | id u64, cursor u64 |
| 10 Contact | id u64, op u8, name, number, label, photo (max 2048) |
| 11 ContactAck | id u64, cursor u64 |
| 12 HistoryPage | cursor u64, count u16, then entries |
| 13 SimList | count u8, then slot u8, label, number |
| 14 Error | code u16, message |
| 15 SyncCursor | collection u8, cursor u64 |

A length-prefixed string or blob is a u16 length plus the bytes. Names and numbers are at most 128 and 32 bytes. History count is at most 64. SIM count is at most 8. Proto in Hello must be 1. Tier must be 0 or 1.

SMS and contact ids are allocated by the sender. The bag phone uses 1, 2, 3, …. The hand phone sets bit 63, so the two phones cannot collide. Receivers deduplicate on id. A resend after reconnect does not create a second row.

Folder 0 is inbox. Folder 1 is sent. Contact op 0 upserts. Op 1 deletes. SIM `0xFF` means "use the remembered SIM for this number, else slot 0". A concrete SIM is remembered for that number.

### Calls

CallInvite creates `ringing`. Direction 0 is a cellular call arriving at the bag phone. Direction 1 is a call the hand phone wants the bag phone to place.

| From | State byte | To |
| --- | --- | --- |
| ringing | 1 active | active |
| held | 1 active | active |
| active | 2 held | held |
| ringing | 4 declined, 5 busy, 6 failed, 3 ended | ended |
| active or held | 3 ended, 6 failed | ended |

Anything else is rejected and the call stays as it was. DTMF (`0-9`, `*`, `#`, `A-D`) is accepted only while active or held. When a call ends, both sides append one history row:

| Status | Meaning |
| --- | --- |
| 0 | incoming, answered |
| 1 | incoming, missed |
| 2 | declined |
| 3 | busy |
| 4 | failed, or outgoing that never connected |
| 5 | outgoing, answered |

Reason 7 together with state 3 is "link lost". The phones end every live call locally, stop audio, and show out of range. A later reconnect does not revive the call id.

### Audio

Audio frames are played only when both Hellos advertised tier 1 and the call is active or held. Otherwise the receiver does not play them and replies with error code 6, `audio_unavailable`. The session stays up.

### Errors

| Code | Name |
| --- | --- |
| 2 | replay |
| 3 | expired QR |
| 4 | reused token |
| 5 | not paired |
| 6 | audio unavailable |
| 7 | unknown type |
| 8 | bad state |
| 9 | bad version |
| 10 | counter ahead |

Local-only failures (truncated frame, bad key, bad tag) use the same names in the reference crate and are not always sent to the peer. A bad tag tears the session down instead of sending a polite error.
