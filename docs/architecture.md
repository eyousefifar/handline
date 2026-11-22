# Architecture

```
handline-core (Rust)     reference frames, handshake, call and SMS behavior
ios/HandlineKit (Swift)  same bytes, checked against testdata/reference.json
android/protocol         same check, Kotlin on the JVM
android/logic            call book, SIM memory, helper install, audio probe
android/app              Compose shell over that logic
ios/App                  SwiftUI shell over HandlineKit
```

The Rust crate is authoritative. The phone libraries do not invent a second frame format. A fixture generated from a fixed seed is committed, and both phone libraries must match it.

## Processes

The bag phone holds the SIM, the identity key, the QR offer, SMS, contacts, and the cellular call. The hand phone holds the bond, a copy of the synced records, and the screen the person is looking at. Traffic keys live only in the session and are wiped on disconnect, replay, or a bad tag.

There is no relay. Pairing does not contact a server. Builds do not embed an analytics SDK.

## Reconnect

`disconnect` keeps the bond and ends live calls. `reconnect` runs the handshake with a zero token and the bonded static keys. The first application frame each way is Hello again. Sync is a resend of rows the other side may already have. Dedup is by sender id.

## SIM choice

Each number remembers a SIM slot. `0xFF` on an invite means "use the memory". The resolved slot is what goes on the wire after the sender applies its memory, and the receiver stores that slot too.

## Identifiers

Bag-phone ids start at 1. Hand-phone ids are the same counter with bit 63 set. History rows use the call id, so both phones append the same row when they apply the same terminal state.

## UI shells

The shells are thin. They render link status, the QR or the scanner, the message list (codes marked), contacts, a keypad, and the helper-code screen. They do not implement cryptography themselves. System call UI on iOS is a `CallPresenting` boundary: the tested book says when a call is ringing, and a CallKit adapter (compiled with the iOS SDK) is the production presenter. Tests use a fake presenter, so they do not need a radio or a SIM.
