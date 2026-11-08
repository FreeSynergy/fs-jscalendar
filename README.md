# FreeSynergy JSCalendar — the same time for everyone

**A building block that makes a calendar entry and a task agree on what a Tuesday at nine means.**

FreeSynergy JSCalendar is a small library with the parts that events and tasks share: time zones, durations, repeat rules, participants, checking, and conversion from and to iCalendar. Without it, the calendar and the task program would each keep a copy and slowly drift apart. It is not a program. It holds no data.

Both owners keep what is theirs: Calendar owns the event, Tasks owns the task. Only what both really use lives here.

## What it can do today

Latest tag: `v0.1.3`.

- Time zones, with the newest tz database found (system or bundled), durations, local and UTC date-times.
- Shared types for participants and repeat rules, with unknown properties kept.
- Upgrade from RFC 8984 to JSCalendar 2.0, safe to repeat, dropping nothing.
- Validation that reports all problems at once.
- Split, read and write iCalendar through a pinned `calcard` version; the original text per entry stays byte for byte.
- Used by FreeSynergy Calendar and FreeSynergy Tasks.

## What is not there yet

- No events or tasks of its own, no storage, no interface, no repeat expansion. Those stay with their owners.
- Only 17 tests live here; most of the behaviour is also covered by the tests of both owners.

## Part of FreeSynergy

This library exists so that two programs share one standard instead of two copies. FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy/fs-documentation); the reading version is at <https://freesynergy.github.io/fs-documentation/>. Start with the [manifesto](https://freesynergy.github.io/fs-documentation/manifesto/) if you want to know why we do this.


## For developers

- Workspace crate: `fs-jscalendar`. The macros `calendar_object!` and `typed_object!` give an object its members and common methods.
- `calcard` is pinned to an exact version on purpose: later versions changed licence to AGPL, which we cannot use.
- Build and test: `cargo build --workspace` and `cargo test --workspace`.
- Design: [DESIGN.md](DESIGN.md).
- Licence: MIT OR Apache-2.0.
