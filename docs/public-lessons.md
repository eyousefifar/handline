# Lessons from public pages

Read on 2026-11-22 from the public site only: the home page, how-it-works, security, download, install, Android setup, the phone list, and two posts (one-time codes, bag-phone battery). No app binary was downloaded. Nothing was decompiled. Sentences below are our notes, not their copy.

Their wire format is not on those pages. Handline keeps the frame layout in `docs/protocol.md`. Named algorithms they do publish (P-256, HKDF-SHA256, AES-256-GCM, a per-packet counter, a five-minute QR token) were already the brief for our own construction.

## Already in this repo

- Android 12+ holds the SIM. iOS 17+ is the phone in the hand. Bluetooth only, about one room.
- Pair once, in person, with a public key and a five-minute one-time token. A later connection makes new ephemeral keys. Packets are authenticated and not replayable.
- The hand phone has no account. Contacts stay inside the app.
- Texts, history, and call state are the floor. Two-way call audio is a probed extra.
- A dropped link ends the live call, shows out of reach, and reconnects without reviving that call.
- English and Persian, with Persian right to left.

## Adopt

These change how the apps should behave. They do not change the frame format.

1. **The bag phone is quiet.** The hand phone rings, including on the lock screen, with the contact name. The bag phone does not play its own ringtone while the link is up. The owner also sets that phone to silent.
2. **The hand phone may have its own SIM.** Handline never places, answers, or texts on that line.
3. **A code is a notification, not only a flag.** When a text looks like a one-time code, the hand phone posts a local notification so the owner can read it with the bag closed. Both phones should have a screen lock. The code is not forwarded to email or another app.
4. **Out of reach is a small strip** that stays until the link returns by itself. Range to tell people: about 10 metres, shorter through walls.
5. **SIM choice has two memories.** A number remembers a slot, and a text thread remembers a slot. The owner can also ask on every call.
6. **Mute and speaker stay on the hand phone.** Mute means the hand phone stops sending uplink audio. Speaker is a local audio route. Neither is a new message type.
7. **Show the QR before any audio helper.** Texts and call alerts do not wait on wireless debugging. The helper is a later, optional step on the bag phone.
8. **Collect the six-digit code without covering the system window.** Android's pairing window must stay open, and a new window means a new code. The field belongs in a notification the owner can pull down over Settings. The in-app screen is the fallback when that notification is not allowed.
9. **Wireless debugging is a Wi-Fi step.** Mobile data is not enough. USB debugging stays off. The owner taps the row title, then "Pair device with pairing code", not the QR row on that system page. An Open button should land on that screen. Leave wireless debugging on afterwards. Ask for the six digits again only if the paired-device entry is gone, the app is reinstalled, or the phone is reset. After a reboot, if the owner left it on, the helper should return when Wi-Fi is back.
10. **Say where the switches hide.** Build number is under About phone, or one level deeper on Samsung and Xiaomi. Developer options may sit under System, Additional settings, or the main Settings list. Searching Settings is the fallback.
11. **Keep the process alive on purpose.** A connected-device foreground service with a visible status notification, plus one ask for unrestricted battery. On Samsung, point at Sleeping apps. On Xiaomi and similar, point at Autostart. Do not claim a vendor intent always exists.
12. **A sound check reports direction, not words.** If a later build places a test call, the only stored result is which way audio opened. The recording is not uploaded.
13. **Unpair is one tap on the bag phone.** It deletes the bond. A replacement phone scans a new QR. There is nothing in a cloud to wipe.
14. **If support logs are ever added,** the owner presses send, the phone strips numbers and message bodies first, and the copy is deleted on a short timer. v1 does not send logs.

## Their published audio notes

These are their claims from 2026-11-22, not Handline measurements. Do not copy the list into a compatibility page.

- They describe two-way audio on a Pixel 8a, a Galaxy A54, and a Nothing Phone (1), plus a longer customer list on their phones page.
- They describe a OnePlus 5T as listen-only on the download page.
- They describe a Galaxy A32 as using one sound method, with slower Bluetooth.
- They describe a Galaxy A24 as needing Wi-Fi calling off.

Our hardware checklist stays "not run" until we place the call. A listen-only result and a Wi-Fi-calling conflict are outcomes the probe should be able to name.

## Not taken

- An account, an email, a license seat, or a server that stores a public key and a phone model. Their media path has no server. Their account path does. Handline has neither.
- A required update that pauses the hand phone after a deadline.
- Their name, icons, screenshots, binaries, file hash, and signing certificate.
- Any packet layout, helper binary, or privilege they did not publish.
- Turning off Play Protect as a setup step. People build this app from source.
