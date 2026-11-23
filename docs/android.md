# Android

Minimum SDK 31. The app is Kotlin. Cryptography and the call book are plain JVM code so unit tests do not need a device or an emulator.

## Permissions, and why

| Permission | Why |
| --- | --- |
| `BLUETOOTH_CONNECT`, `SCAN`, `ADVERTISE` | The L2CAP link to the hand phone |
| `FOREGROUND_SERVICE` and `CONNECTED_DEVICE` | Keep the link while the phone is in a bag |
| `POST_NOTIFICATIONS` | The ongoing "linked" notice, and the one-time code entry |
| `READ_SMS`, `RECEIVE_SMS`, `SEND_SMS` | Show and send the bag phone's texts |
| `READ_CONTACTS`, `WRITE_CONTACTS` | Names on calls, and contacts added from the iPhone |
| `READ_CALL_LOG` | History |
| `READ_PHONE_STATE`, `CALL_PHONE` | Which SIM, and placing the call the iPhone requested |
| `REQUEST_IGNORE_BATTERY_OPTIMIZATIONS` | Ask, once, not to be frozen in a pocket |

The manifest does not request `CAPTURE_AUDIO_OUTPUT`. That permission is not grantable to a normal app, and asking for it does not make audio work.

## Services

A connected-device foreground service owns the Bluetooth socket. SMS and call events are delivered into the protocol endpoint. The service does not start at boot until the owner has finished pairing and left the link enabled.

## OEM battery screens

Some phones still freeze a foreground service. The in-app copy points at the system battery screen and does not pretend a vendor-specific intent always exists. On Samsung it also points at Sleeping apps. On Xiaomi and similar phones it points at Autostart. If the link drops because the process was frozen, the hand phone shows out of range and reconnects when the process runs again.

The status notification stays visible while the link is enabled. That is what keeps the process in a bag. The bag phone does not start its own ringtone for a call the hand phone is presenting. Setup buttons open About phone, Developer options, and Wireless debugging. Searching Settings remains the written fallback.

## What the unit tests cover

The JVM tests cover the fixture, the call book, SIM memory, one-time-code detection, the helper install state machine, and the audio probe. They do not place a cellular call.
