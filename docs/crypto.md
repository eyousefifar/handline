# Cryptography

Primitives are the ones named in public descriptions of this product category, used in an original construction:

- P-256 ECDH, SEC1 compressed public keys
- HKDF-SHA256
- AES-256-GCM with 12-byte nonces and 16-byte tags
- SHA-256 for the handshake transcript

Identity keys are generated on the phone. The long-term secret stays in the Android Keystore or the iOS Keychain in the app builds. The reference crate keeps it in memory so tests can inject scalars. A test vector is not a production key.

## What a nearby radio sees

Bluetooth advertisements and L2CAP payloads. Payloads after Finished are AES-GCM ciphertext. The header (version, type, flags, counter, length) is visible and covered by the GCM additional data, so a flipped bit fails the tag. The handshake public keys and nonces are visible. The pairing token is not. Without both static private keys and the token used for that handshake, the traffic keys are not recoverable from the radio.

## Forward secrecy

Each connection generates new ephemeral keys. A later leak of the long-term identity keys does not open a finished transcript from an earlier connection. The token is mixed into the salt so a QR that was never scanned does not become a reusable authenticator.

## Replay and reorder

Each direction has its own counter. Counter 0 is only the Finished message. Application counters start at 1. The receiver accepts a counter inside a 32-wide window and rejects a repeat. A counter further ahead than the window, or a bad tag, deletes the traffic keys. The bond remains, so the phones can handshake again. Live calls are marked ended.

## Pairing assumptions

The QR code is the out-of-band channel. Anyone who can photograph it during its five-minute life can attempt the handshake. The product does not pretend otherwise. After the bond exists, a third static key is rejected on reconnect and the current session is left in place.

## One-time codes

`is_otp` is true when the trimmed body is 4 to 8 ASCII digits, or when the body contains `code`, `otp`, `verify`, or `verification` (any case) and a run of 4 to 8 digits. This is a local hint so the hand phone can surface a code. It is not authentication.

## Deliberate non-goals

No account key is uploaded. No license signature is checked. No certificate pinning against a vendor. Diagnostic logs are not implemented; if they are added later they must be opt-in and must strip numbers and message bodies before anything leaves the phone.
