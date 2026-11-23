# Product

Handline puts the calls, texts, and contacts of an Android phone (the bag phone) on a nearby iPhone (the hand phone). The phones talk over Bluetooth. There is no server in the media path and no account.

This repository is an independent implementation. It is not iTell and it is not affiliated with itellapp.com. Behavior below is the contract for these apps. Public product pages were read as a category brief (`docs/public-lessons.md`). The tests in this repo pin the behavior. Their packet layout was not published and is not used.

## People

Someone carries an iPhone and keeps a second SIM in an Android phone in a bag or the same room. They want that number's calls and texts on the iPhone, including a one-time code, without carrying the Android phone in their hand.

## What v1 does

- Pair once by scanning a QR code. The code expires in five minutes and works once. The QR appears after link permissions, and does not wait on the call-audio helper.
- Show incoming calls on the hand phone, including the lock screen, with the contact name. Answer, decline, send a keypad digit, mute, switch to speaker, hang up. Mute stops uplink audio. Speaker is a local route.
- While the link is up, the bag phone does not ring. The owner also leaves that phone silent.
- Place a call from the hand phone. The bag phone is the phone that dials. The hand phone's own SIM, if it has one, is left alone.
- If the bag phone has two SIMs, remember which SIM a number used and which SIM a text thread used, and allow an override or a prompt every time.
- Sync SMS both ways. A message that looks like a one-time code is flagged and posted as a local notification on the hand phone.
- Sync contacts inside the hand app. Do not write them into the iOS Contacts database.
- Keep a call history with missed calls distinct from answered and declined calls.
- Show a small out-of-reach strip and reconnect without reviving a dead call. Tell people to expect about 10 metres, less through walls.
- Unpair from the bag phone in one tap. That deletes the bond. A new phone needs a new QR.
- English and Persian strings. Persian layout is right to left.

## What v1 does not do

- No account, license, payment, or forced update.
- No upload of messages, contacts, call audio, or location.
- No mixing of bag-phone contacts into the iPhone's system contacts.
- Two-way in-call audio only when a local probe can open the voice-call path. Otherwise the call is an alert plus call controls, and the app says so.

## Tiers

Tier A is every Android 12 phone. Texts, contacts, history, and call state work. Call audio is not claimed.

Tier B is opt-in. The owner types the six-digit wireless-debugging code shown on their own screen. A helper this repo builds is installed only after that. A probe then tries the platform audio sources. Two-way audio is reported only if the voice-call source opens. Microphone-only is not called two-way. A failed probe leaves the phone on Tier A.

## Out of range

When the link drops, live calls end, audio stops, and the hand phone shows that the bag phone is out of reach. The phones handshake again when the link returns. Old call ids stay ended.
