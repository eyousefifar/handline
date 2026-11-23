# Docs map

The docs are the contract. Code that disagrees with them is wrong. `design/ui/` is the screen reference the apps follow later.

## What the product is

| File | Holds |
| --- | --- |
| `product.md` | Who it is for, what v1 does, the two tiers |
| `public-lessons.md` | What the public category pages taught, and what we refused |
| `architecture.md` | Which process owns the SIM, the bond, and the screen |
| `ui.md` | Every screen, in order, and the states that change them |
| `android.md` | Permissions, the foreground service, battery copy |
| `ios.md` | CallKit boundary, storage, language |
| `call-audio.md` | The helper, the probe, and what Tier A must say |

## What the bytes are

| File | Holds |
| --- | --- |
| `protocol.md` | Frames, messages, call states, error numbers |
| `crypto.md` | Keys, handshake, what a listener can see |
| `threat-model.md` | Who we stop, and who we do not pretend to stop |
| `test-plan.md` | What CI proves, and the hardware rows still marked not run |

## Decisions

`adr/0001` Rust is the reference. `0002` the link is L2CAP. `0003` no account. `0004` the name is Handline.

## Agents

`agents.md` says who may edit which directory. Prompts live in `agents/prompts/`. A prompt that does not name a path does not grant that path.
