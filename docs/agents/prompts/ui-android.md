# Android shell prompt

You may edit only `android/app/**`. You may also add `apply false` plugin lines to `android/build.gradle.kts` so the app module can apply the Android and Kotlin Android plugins. Do not give those plugins to `:protocol` or `:logic`. Do not add a dependency those two modules need.

Read `docs/ui.md`, `docs/ui-direction.md`, `docs/android.md`, and `design/ui/index.html`. Compose copies those screens. The bag phone is the dark instrument. The hand-phone screens are not this module.

Do not change `settings.gradle.kts`. The app module is included only when an Android SDK is present. `./gradlew test` from `android/` must keep passing with no SDK, because `:app` is not on the project then.

Permissions stay the list in `docs/android.md`. Do not add `CAPTURE_AUDIO_OUTPUT`. The manifest does not start the service at boot before pairing. This pass is the UI. Do not open a Bluetooth socket and do not start a foreground service. A comment in the manifest is enough to show the service is absent until pairing exists.

Visual rules:

- Dark Material 3 scheme locked to the bag tokens in `docs/ui-direction.md`. Dynamic color off, so the system cannot invent a second accent.
- `MotionScheme.standard()`. No bounce on color or opacity. No row entrance animation. No Lottie.
- The filament is a stroke meaning the link is up. It is not a button tint, not a code color, and not "sound is ready".
- Platform sans. Persian follows layout direction. Digits, the pairing clock, and the six-digit field stay left to right.
- Lists and rows, not cards. Press scale 0.98 only on buttons. The first screen does not animate in.

Screens, with sample state for linked and out of reach, and for texts-only versus two-way sound: permissions, pairing code, home, call sound, unpair. English and Persian.

The four tested strings stay character for character, including Persian, as listed in `docs/ui-direction.md`. Call sound copy says texts and ringing still work when the probe fails.

Do not edit `android/protocol/**`, `android/logic/**`, the Rust crate, or iOS. Do not download an APK, run a decompiler, or copy another product's screens, helper, or strings. Do not commit.
