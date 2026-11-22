# Agents

Grok holds the contract: this document set, the Rust reference, and review. Droid (`droid exec -m glm-5.3-flash`) is for bounded UI and boilerplate after a spec file exists. Flash does not design the frame format, the key schedule, or the threat model.

| Wave | Who | Owns | Done when |
| --- | --- | --- | --- |
| Spec | Grok | `docs/**`, ADRs | byte tables match the crate |
| Reference | Grok | `handline-core/**` | `cargo test` green, fixture exported |
| Swift | Grok | `ios/HandlineKit/**` | `swift test` matches the fixture |
| Kotlin | Grok | `android/protocol`, `android/logic` | `gradle test` matches the fixture |
| Shells | Droid, reviewed by Grok | `android/app`, `ios/App` | compiles against the logic API, no protocol edits |
| Device | a person | hardware table | rows filled from a real run |

Prompts for a shell agent live in `docs/agents/prompts/`. An agent may edit only the paths in its prompt. It may not download an APK, run a decompiler, or copy another app's screens.

Review before merge is a diff against `docs/protocol.md`. A frame, a constant, or a permission that the spec does not list is rejected.
