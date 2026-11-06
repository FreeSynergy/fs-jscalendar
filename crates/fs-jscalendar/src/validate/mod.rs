//! Validation rules every JSCalendar object shares: module map.
//!
//! Pattern: Facade (declarations) — owners add their type-specific rules to a [`Ctx`] and call
//! [`common`] and [`shared`] for the rest.

mod ctx;
mod rule;
mod shared;

pub use ctx::{Ctx, into_result};
pub use shared::{common, shared};
