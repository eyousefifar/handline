# UI direction — pocket filament

This file wins over taste. Screens, order, and wording stay in `docs/ui.md`. Tokens and motion live here. The HTML reference at `design/ui/index.html` is the picture of those screens. SwiftUI and Compose copy the screens, the order, and the words. Chrome is the platform's, described under Native.

## The one idea

Handline is a line between a phone in a bag and a phone in a hand. That line is a filament, two points wide. It is the only decoration. Everything else is quiet.

The bag phone is a dark instrument, glanced at in a pocket. The hand phone is a daylight reader, used all day. They share the filament and the words. They do not share a card kit.

## Tokens

| Name | Value | Use |
| --- | --- | --- |
| bag | `#161A1E` | Bag phone ground |
| bag-text | `#E7EDF2` | Bag phone text |
| bag-dim | `#9AA4AE` | Bag phone secondary |
| hand | `#F3F5F7` | Hand phone ground |
| ink | `#172028` | Hand phone text |
| dim | `#5C6770` | Hand phone secondary |
| hairline | `#D5DCE2` | Row rules on the hand phone |
| filament | `#E2A13A` | The live link only. A stroke, never a fill and never a gradient |
| missed | `#C4362C` | Missed name, Decline, Unpair. Nowhere else |
| press | scale `0.98` | Buttons and keypad keys, 90ms |

Do not introduce a second accent. Do not tint buttons, labels, focus rings, tab indicators, or code marks with the filament. A code is marked by weight. Focus and the tab tick use ink on the hand phone and bag-text on the bag phone. "Sound is ready" is bag-text, not gold.

The filament is an inline SVG stroke, the same curve as the studio mark, with a dot at each end. It draws by `stroke-dashoffset` (or the platform equivalent on a path). It is not a bar whose width changes.

## Type

System faces. San Francisco on iOS. The platform sans on Android. Persian uses the platform Arabic face. Do not load a webfont, a Latin display family, or a Persian display family.

Code digits, the dialed number, the pairing clock, and the lock clock are tabular and stay left to right in both languages.

One hero per screen. On the bag home, the hero is the status sentence. On a code, the hero is the digits. On an incoming call, the hero is the name.

## Motion

Ease-out for anything entering: `cubic-bezier(0.23, 1, 0.32, 1)`. Animate transform and opacity, or a stroke offset. Do not animate width, height, or padding. The first screen on load is still.

| What | Duration | Curve | Reduce Motion |
| --- | --- | --- | --- |
| Filament draws when the link becomes up | 700ms, once | ease-out | a static tick, no draw |
| Filament retracts when the link drops | 400ms | ease-in | the tick disappears |
| Pushed screen | 280ms, 16px and fade | ease-out | opacity 120ms |
| Call sheet | 420ms rise | stiff, no overshoot | opacity 120ms |
| Press | 90ms to scale 0.98 | linear | none |
| Tab tick | 180ms slide | ease-out | instant |
| Out-of-reach strip | transform in once | ease-out | opacity only |

Press scale applies to primary buttons, round call buttons, and keypad keys. List rows do not scale.

No entrance animation on rows. No stagger. No bounce. No Lottie. No looping motion. The filament draws once per link change and then holds.

Native shells use the system sheet and the system navigation transition for the call sheet and pushed screens. The durations above are the HTML reference. Android uses Material 3's standard motion scheme (high damping, no bounce on color or opacity). iOS uses SwiftUI transitions and drops them when `accessibilityReduceMotion` is on.

## Pictures

`design/ui/assets/` holds samples, not user content. One photographic grade. No words, logos, screens, or codes in the frame. `bag-near.jpg` and `bag-far.jpg` are the same bag.

| File | Frame | Where it shows |
| --- | --- | --- |
| `bag-near.jpg` | 3:2 | Pairing, behind the code. Inside a dark canvas bag, phone face-down, close, warm light through a zipper gap. Darken with a scrim so the title and the timer clear 4.5:1 |
| `bag-far.jpg` | 3:2 | Out of reach on the bag home, 132px tall, corner radius 16. The same bag, farther, cooler, less light |
| `sara.jpg` | 1:1 | Round avatar. A woman, window light, head and shoulders, center-weighted |
| `cafe.jpg` | 1:1 | Round avatar. A cafe table and a cup. No signage |

The bank is a word, not a face. The app mark is an inline SVG of the filament, not a generated image. Pictures do not appear on texts as heroes, on the keypad, on recents as heroes, or on the call sheet. Avatars on those lists are the exception.

## Native

The HTML page is a review bench with two phone frames. The apps are not a copy of that bezel.

iOS, deployment target 17. `TabView` and `NavigationStack`. Lists, not cards. SF Symbols. Dynamic Type. On an iOS 26 SDK the system tab bar and toolbar pick up Liquid Glass. Do not call a glass effect on rows, bubbles, or the keypad, and do not imitate glass with a blur. The incoming screen in the reference is the list of facts CallKit must carry. The app does not paint a second call UI over the system one. A labeled preview of those facts is allowed. Contacts are not written to the system address book.

Android. Compose. A dark Material 3 scheme locked to the bag tokens. Dynamic color stays off. `MotionScheme.standard()`. Predictive back. The platform sans. Digits stay left to right. The bag phone is the status surface, not a second inbox.

## Banned

These read as a template. Do not add them.

- Warm cream grounds, terracotta fills, acid green, or a purple gradient.
- Identical rounded cards with a soft grey shadow.
- All-caps tracked labels above headings.
- Middle-dot meta lines (`Linked · quiet`).
- A serif display headline.
- Numbered markers that are not a real sequence. Call sound may be numbered. It is a sequence.
- Arrows glued to buttons.
- Generated pictures of screens, words, or codes.
- A webfont, a Lottie file, or a third-party component kit.

## Words that must survive

From the tested copy map, character for character:

| Key | English | Persian |
| --- | --- | --- |
| linked | Linked | وصل |
| out_of_range | The other phone is out of reach | گوشی دیگر در دسترس نیست |
| audio_unavailable | Call audio is not available on this phone | صدای تماس روی این گوشی در دسترس نیست |
| pair | Type the 6-digit code | رمز شش‌رقمی را وارد کنید |

"Lunch at 1234 Main" is an ordinary text. It is not marked as a code.
