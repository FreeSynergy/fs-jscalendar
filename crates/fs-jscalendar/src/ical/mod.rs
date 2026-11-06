//! iCalendar (RFC 5545) to JSCalendar and back: module map.
//!
//! Pattern: Adapter (declarations) — the files wrap `calcard`; owners get typed objects from JSON
//! values, so each keeps only its own type here.

mod convert;
mod split;

pub use convert::{ICS_MEDIA_TYPE, parse_entries, write_ics};
pub use split::{Component, IcsPart, split_ics};
