//! Error type of the shared JSCalendar types.
//!
//! Pattern: none (plain error enum via `thiserror`) — owners turn it into their own error with one
//! `From`, keeping the same variants.

/// One validation problem with a path into the JSCalendar object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    /// JSON-pointer-like path, e.g. `/recurrenceRule/count`.
    pub path: String,
    /// What is wrong.
    pub message: String,
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Result of every fallible operation of this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong with the shared types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The object violates JSCalendar 2.0 (all problems listed).
    #[error("JSCalendar 2.0 violation: {}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join("; "))]
    Invalid(Vec<Issue>),
    /// A date-time, duration or time zone could not be handled.
    #[error("time: {0}")]
    Time(String),
    /// iCalendar text could not be read or written.
    #[error("iCalendar: {0}")]
    ICalendar(String),
    /// JSON could not be read or written.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}
