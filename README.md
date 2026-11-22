# Handline

Handline is a phone-to-phone bridge. An Android phone keeps the SIM. An iPhone in your hand gets that phone's calls, texts, and contacts over an encrypted Bluetooth link. There is no account and no server in the path.

This is independent software. It is not iTell, and it is not affiliated with [itellapp.com](https://itellapp.com). The protocol, the code, and the words are original. See `docs/adr/0004-name.md`.

## What works in this repository

The reference implementation and the two phone libraries are tested without a radio:

| Check | Command |
| --- | --- |
| Rust protocol, calls, SMS, replay, QR | `cargo test --manifest-path handline-core/Cargo.toml` |
| Swift against the same fixture | `swift run --package-path ios/HandlineKit HandlineCheck` |
| Kotlin against the same fixture, plus the helper state machine | `./gradlew test` in `android/` |

`handline-core/testdata/reference.json` is the byte contract. Swift and Kotlin have to reproduce it.

Tier A is texts, contacts, history, and call state. Tier B, two-way call audio, is opt-in and only after a probe opens the platform voice-call source. A normal app is not expected to get that source. The phone stays on Tier A and says so. Details are in `docs/call-audio.md`.

## Layout

```
docs/                 product, protocol, threat model, tests, ADRs
handline-core/        Rust reference and the scenario tests
android/protocol      Kotlin handshake and frames
android/logic         call book, SIM memory, helper install, copy
ios/HandlineKit       the same checks in Swift
ios/App               SwiftUI shell, needs a full Xcode install
```

## Build

Rust 1.80 or newer, a JDK 17, and the Swift tools that ship with Xcode or the Command Line Tools.

```
cargo test --manifest-path handline-core/Cargo.toml
swift run --package-path ios/HandlineKit HandlineCheck
export JAVA_HOME=/opt/homebrew/opt/openjdk@17   # or any JDK 17
cd android && ./gradlew test
```

The iOS app target and the Android `app` module are compiled only when the iOS SDK and `ANDROID_HOME` are present. They are not required for the tests above. Hardware results are in `docs/test-plan.md` and are not filled in until a phone is actually used.

## License

Apache-2.0. See `LICENSE`.
