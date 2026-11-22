# Call audio

Android does not let an ordinary app read or inject the sound of a cellular call. Tier A does not pretend otherwise. The hand phone still gets the ring, the answer, the decline, the digits, and the hangup. The people on the call use the bag phone's earpiece unless Tier B is ready.

## Tier B

Tier B is off until the owner does three things on the bag phone:

1. Turn on Developer options and Wireless debugging, on Wi-Fi.
2. Open "Pair device with pairing code" and read the six digits. The window has to stay open.
3. Type those digits into Handline.

The install state machine:

| Phase | Allowed event | Next |
| --- | --- | --- |
| idle or failed | six digits | pairing |
| idle or failed | anything else | stay, error |
| pairing | pair succeeded | installing |
| pairing | pair failed | failed |
| installing | install succeeded | probing |
| probing | voice-call source opened | ready |
| probing | only mic, or nothing | failed, phone stays Tier A |

Events from the wrong phase are ignored. Install does not run before pair. The code never leaves the phone. There is no remote install and no silent install.

## Probe

The probe asks an opener, in order, for `VOICE_CALL`, then `VOICE_COMMUNICATION`, then `MIC`.

| First source that opens | Result |
| --- | --- |
| `VOICE_CALL` | two-way, Tier B ready |
| communication or mic | uplink only, not two-way, Tier A |
| none | Tier A |

A normal install is expected to fail `VOICE_CALL`. The app then says two-way audio is unavailable on this phone. That is a successful, honest result.

The helper is source in this repository. It is not a binary taken from another product. Uninstalling it, or a failed probe, leaves Tier A working.

## On the air

Opus at 16 kbit/s, 20 ms, mono, one frame per Audio message, only while the call is active or held and both Hellos said tier 1. If the link cannot carry that, the phones stop audio and keep the call state. They do not switch to an undocumented codec.
