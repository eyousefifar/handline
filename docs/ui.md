# UI reference

Open `design/ui/index.html` in a browser. That page is the screen contract. Tokens and motion are in `docs/ui-direction.md`. SwiftUI and Compose copy its screens, its order, and its wording. They do not invent a second information architecture.

The page is a picture. It does not pair, dial, or store anything. Sample names and codes are fake.

## Two apps, two jobs

The bag phone is a status surface. Its home shows whether the link is up, that this phone stays quiet, and whether call sound opened. It is not a second inbox.

The hand phone is the inbox. Texts, the keypad, history, and contacts live there. Contacts never go into the iPhone address book. The hand phone's own SIM is never the line Handline dials.

## Bag phone

| Screen | Must show |
| --- | --- |
| Permissions | Each grant in one line, with the reason. Continue does not install the audio helper. |
| Pairing code | A code the other phone scans. Once, five minutes. Pairing does not wait on call sound. |
| Home | Linked and quiet, or out of reach. A separate row for call sound. Unpair is here, not buried. |
| Call sound | Wi-Fi, the pairing-code row rather than the system QR row, the window stays open, six digits typed in a notification. |
| Result | Tier A says texts and ringing still work and voice stays on this phone. Tier B says both directions opened. |
| Unpair | One confirm. The bond is deleted. The other phone needs a new code. |

## Hand phone

| Screen | Must show |
| --- | --- |
| Scan | The camera aimed at the bag phone. No account. |
| Texts | Threads, the SIM each thread remembers, a code marked only when the detector says so. "Lunch at 1234 Main" is not a code. |
| Thread | Reply field. Replies leave from the bag phone. The field waits while the link is down. |
| Keypad | Digits, the remembered SIM, a way to ask every time. Copy says the iPhone line is not used. |
| Recents | Missed is visually distinct from answered and outgoing. |
| People | A line that these contacts stay inside Handline. |
| Incoming | Name, SIM, answer, decline. This is the information the system call screen carries. |
| In call | Mute, speaker, keypad, end. Mute stops uplink. Speaker is local. Tier A says voice stays on the Android phone. |
| Code notification | A one-time code on the lock screen, readable with the bag closed. |
| Out of reach | A strip on the texts screen. Live calls end. The strip leaves when the link returns. |

## Language

English is left to right. Persian is right to left, including the tab bar and the rows. The keypad and the dialed number stay left to right. The four strings already tested in `copy()` stay as they are: linked, out of reach, call audio unavailable, and the six-digit prompt.

## What the native apps do not redraw

The iOS lock-screen call is CallKit. The HTML incoming screen is the list of facts that screen must carry, not a skin to paint over the system UI.
