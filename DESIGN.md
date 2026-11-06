# fs-jscalendar – Design

The parts `Event` (owner: `fs-calendar`) and `Task` (owner: `fs-tasks`) have in common, so neither copies them (ADR 0043 §4).
ADR 0050 allows exactly this: shared **types** of a standard may live in a plain library. It holds no data, stores nothing,
has no interface and is not a program. The point of truth for each object stays with its owner.

**What does not belong here:** `Event`, `Task`, repositories, the invitation flow and recurrence expansion (`fs-calendar`), relations and
task status (`fs-tasks`). A type moves here only when both owners really use it.

## State

Version 0.1.3. Taken from `fs-tasks-core` and `fs-calendar-core`, which held identical copies.

| Area | State |
|---|---|
| `time`: `Zone`, `ZoneSource` (system / bundled tzdb, newest wins), `Dur`, `LocalDateTime` / `UTCDateTime` | done |
| `model`: `Participant`, `RecurrenceRule`, `NDay`, aliases, constants, `next_participant_key` | done |
| `calendar_object!` / `typed_object!`: members and common methods of an object, lossless `extra` | done |
| `upgrade`: RFC 8984 to 2.0, idempotent, nothing dropped; fractions leave every date-time and duration of the object, durations round up; values in alerts and participants stay as they are | done |
| `validate`: `Ctx`, `common`, `shared`, recurrence rule; owners add their own rules | done |
| `ical`: `split_ics(text, Component)`, `unfolded_lines`, `parse_entries`, `write_ics` through `calcard =0.3.14`; components are found on unfolded lines, so a continuation line never ends one | done |
| Date-times: exactly `YYYY-MM-DDTHH:MM:SS`, second 60 refused; messages quote a value short and escaped (`quoted`) | done |
| Tests | 17 here; the behaviour is also covered by the tests of both owners |

## Structure and pattern map (ADR 0043)

| Module | Pattern | Why |
|---|---|---|
| `model/macros.rs` | Template method (macro form) | fixes the skeleton of an object; owners list only their own members |
| `model/{participant,rule}.rs` | Value object | plain serde data, lossless `extra` |
| `time/source.rs`, `bundled.rs`, `system.rs` | Strategy + Template method | `ZoneSource`: where tz rules come from |
| `time/select.rs` | Factory | picks the newest of system and bundled tzdb at first use |
| `time/{zone,dur,format}.rs` | Value objects | `Zone`, nominal-day + exact-second `Dur`, wire formats |
| `validate/*` | Specification + Collecting parameter | rules report into one `Ctx`; all issues at once |
| `upgrade.rs` | Pipeline | small rewrite steps, idempotent, with notes |
| `ical/convert.rs` | Adapter | wraps `calcard`; passes JSON values, owners make them typed |
| `ical/split.rs` | Iterator-style scan | exact source text per UID, so the original stays byte for byte (ADR 0030) |
| `upgrade_times.rs` | Pipeline step | whole seconds for `upgrade`, every original kept |
| `error.rs`, `clock.rs`, `uid.rs` | none | plain error enum and pure helpers |

## Decisions

- **Errors:** owners turn `fs_jscalendar::Error` into their own with one `From` and keep the same variants.
- **`calcard` is pinned to `=0.3.14`:** the last MIT/Apache version before later releases moved to AGPL.
- **`uid`s without `fs-store`:** this crate stores nothing, so it does not depend on the store. `new_uid` makes the same `urn:uuid:` + UUIDv7 form.
