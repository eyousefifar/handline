# Test plan

A green CI run means the protocol and the call book behave. It does not mean a cellular call was placed. Hardware is a separate checklist and stays unchecked until a person runs it.

## Layer 1 — reference crate

`cargo test` in `handline-core`.

- Incoming answer, DTMF, hangup, and both sides agree.
- Decline leaves the bag phone on hook.
- A hangup while ringing is a missed call.
- SIM choice sticks to the number, including the default `0xFF`.
- SMS in both directions, a code is flagged, a street address is not, a resend after reconnect does not duplicate.
- Contact add, photo replace, delete.
- QR older than five minutes is refused. A second hello with the same code is refused.
- A flipped handshake byte does not install keys.
- A third static key is refused and the live session stays.
- A replay kills the session and the call.
- A counter 32 ahead is rejected. A counter 31 ahead is accepted, and the skipped counter still is too.
- Out of range ends audio. Reconnect does not resume that call.
- Tier A does not play audio and reports error 6. Tier B does play it.
- Random bytes into the frame sniffer and the QR parser do not panic.

## Layer 2 — other languages

`swift test` and `gradle test` load `testdata/reference.json` and check the QR, both hellos, both traffic keys, both Finished messages, the Hello plaintext, a replay, and a tampered tag.

The same suites check the call book, SIM memory, the code detector, the audio probe, and the install state machine.

## Layer 3 — shells

Compose and SwiftUI bind to those types. They are not the conformance suite. Building the iOS app needs Xcode. Building the Android app needs an Android SDK.

## Hardware checklist

Not run in CI. This workspace had no Android SDK and no phone on `sixpath@192.168.254.13` (`adb` was not installed there). Do not mark a row from memory.

| Check | Result |
| --- | --- |
| Bag phone model and Android version | not run |
| Hand phone model and iOS version | not run |
| Install, grant SMS, contacts, phone, notifications | not run |
| Pair from a QR | not run |
| Inbound SMS appears on the iPhone and a reply is received | not run |
| Outgoing call from the iPhone UI dials on the bag phone | not run |
| Walk out of range and back | not run |
| Tier B only after the owner types the wireless-debugging code | not run |
| Helper removed, Tier A still runs | not run |

`sixpath` is the place to run `adb` when a phone is plugged in. It is not a place to disassemble someone else's app.
