# iOS shell prompt

You may edit only `ios/App/**`.

Read `docs/ui.md`, `docs/ui-direction.md`, and `design/ui/index.html`. SwiftUI copies those screens, that order, and those words. It does not invent a second layout. Chrome is the platform's, as the Native section of `docs/ui-direction.md` says.

Rules:

- One hero per screen.
- Color is information. The filament is an SVG-or-path stroke meaning the link is up. Missed red means missed, decline, or unpair. Do not tint buttons, tab indicators, focus, or code marks with the filament. A code is marked by weight.
- System controls, San Francisco, SF Symbols. `TabView` and `NavigationStack`. Lists, not cards. Persian follows the layout direction. Digits, the dialed number, and codes stay left to right.
- Dynamic Type. `accessibilityReduceMotion` drops the filament to a static tick and screen changes to a short fade. No row entrance animation, no bounce, no Lottie.
- Deployment target stays iOS 17. Do not call `glassEffect` on content. Do not imitate Liquid Glass with a blur. System bars pick the material up on their own when the SDK is new.
- The incoming screen is a labeled preview of the facts CallKit must carry, matching the HTML note. It is not a custom skin over the system call UI.
- Contacts are not written to the system address book. These files do not touch `CNContactStore`.
- Sample data matches the HTML: Sara, Bank, Cafe. "Your code is 482193" is a code. "Lunch at 1234 Main" is not. The call control is disabled when the link is down.
- The screens render without a radio. Do not open Bluetooth. You may read types from HandlineKit only by duplicating the small sample model inside `ios/App`. Do not edit the package.

Do not edit `Package.swift`, `ios/HandlineKit/**`, protocol code, or Android. Do not add the app target to the Swift package. Do not download an APK, run a decompiler, or copy another product's screens or strings. Do not commit.

The four tested strings stay character for character: "Linked", "The other phone is out of reach", "Call audio is not available on this phone", "Type the 6-digit code", and their Persian lines in `docs/ui-direction.md`.

Screens to include, both languages, with a way to preview Persian, the link down, and texts-only versus two-way sound: permissions are the bag phone's, so the hand app shows scan, texts, thread, keypad, recents, people, the incoming facts preview, in call, the code notification preview, and the out-of-reach strip.
