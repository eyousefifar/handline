# HTML reference prompt

You may edit only `design/ui/**`.

Read `docs/ui.md` and `docs/ui-direction.md`. Implement that direction in the existing page. Do not invent screens, states, or words.

Keep every `data-k` key, both languages in `app.js`, the hash state (`lang/link/audio/bag/hand`), and these screen ids: permissions, qr, home, audio, unpair, scan, texts, thread, keypad, recents, people, incoming, active, code.

The four tested strings stay character for character, in both languages. "Lunch at 1234 Main" has no code mark. The keypad, the dialed number, and code digits stay `dir="ltr"`. The call button is `disabled` when the link is away, not merely faded.

## Craft, this pass

The filament inside each phone is an inline SVG stroke, the same curve as the studio mark, with a dot at each end. It draws with `stroke-dashoffset` over 700ms ease-out when linked, and retracts over 400ms ease-in when away. A short tick of that stroke sits under the Texts title while linked. Do not animate `width`. Under `prefers-reduced-motion`, the stroke is a static tick with no draw.

Filament `#E2A13A` is the live link only. It is not the tab tick, not the focus ring, not the code label, not the notification app name, and not the "sound is ready" line. Those use ink on the hand phone and bag-text on the bag phone. A code is marked by font weight.

Bag ground `#161A1E`. Hand ground `#F3F5F7`. No card shadows, no all-caps eyebrows, no middle-dot status lines, no serif headlines, no second accent, no webfont.

Use `assets/bag-near.jpg` on the pairing screen, darkened so the title and the timer clear contrast. Use `assets/bag-far.jpg` on the bag home when away, 132px tall, radius 16. Use `assets/sara.jpg` and `assets/cafe.jpg` as round avatars. The bank has no portrait. The mark is an inline SVG.

Motion, and nothing else:

- Pushed screens: 280ms, 16px and fade, `cubic-bezier(0.23, 1, 0.32, 1)`. The first screen on load does not play this.
- Call sheet: 420ms rise, stiff, no bounce.
- Press: 90ms to scale 0.98, on primary buttons, round call buttons, and keypad keys only. List rows do not scale.
- Tab tick: 180ms, ink, slides on the inline-start edge so Persian mirrors it.
- Out-of-reach strip: one transform-in. No pulse.
- Reduced motion: opacity 120ms, no movement. The filament tick does not draw.

Animate transform, opacity, or stroke offset only.

Do not edit `docs/`, `handline-core/`, `ios/`, `android/`, or CI. Do not download an APK, run a decompiler, or copy another product's screens or strings. Do not commit.
