//! The parts every JSCalendar object is made of: module map.
//!
//! Pattern: Facade (declarations) — `Event` and `Task` live with their owners and are built with
//! the [`calendar_object!`](crate::calendar_object) macro from these parts. Unknown and vendor
//! properties stay verbatim in `extra`, so round trips lose nothing.

mod consts;
mod macros;
mod participant;
mod rule;

pub use consts::{Extra, LEGACY_KEY, Map, VENDOR, VERSION};
pub use participant::{PARTICIPATION_STATUSES, Participant, next_participant_key};
pub use rule::{NDay, RecurrenceRule};
