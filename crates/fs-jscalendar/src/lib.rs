//! `fs-jscalendar`: the shared types of the JSCalendar 2.0 standard.
//!
//! A plain library: **no data, no storage, no interface, no program** (ADR 0050). The objects
//! themselves belong to their owners, `Event` to `fs-calendar` and `Task` to `fs-tasks`; both build
//! on what is here, so nothing is copied (ADR 0043 §4). See `DESIGN.md` for the pattern map.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// Pedantic lints that would only add noise to this crate's small, documented API:
// every fallible fn returns the crate `Result` (documented once on `Error`), results of pure
// getters are used by callers anyway, and the date arithmetic casts are range-checked by jiff.
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod clock;
mod error;
pub mod ical;
pub mod model;
pub mod time;
mod uid;
pub mod upgrade;
mod upgrade_times;
pub mod validate;

pub use clock::now_utc;
pub use error::{Error, Issue, Result, quoted};
pub use uid::new_uid;

/// What the object macros need at their call site, so owners name no extra dependency for them.
#[doc(hidden)]
pub use serde_json as __json;
