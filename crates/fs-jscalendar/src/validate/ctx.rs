//! Collector for validation problems.
//!
//! Pattern: Collecting parameter — every rule reports into one `Ctx`, so an object is checked
//! completely and the caller sees all problems at once.

use crate::error::{Error, Issue, Result, quoted};

pub(super) const FREQ: [&str; 7] = [
    "yearly", "monthly", "weekly", "daily", "hourly", "minutely", "secondly",
];
pub(super) const DAYS: [&str; 7] = ["mo", "tu", "we", "th", "fr", "sa", "su"];

/// The problems found so far.
#[derive(Clone, Debug, Default)]
pub struct Ctx(Vec<Issue>);

impl Ctx {
    /// An empty collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Report a problem at `path`.
    pub fn bad(&mut self, path: &str, msg: impl Into<String>) {
        self.0.push(Issue {
            path: path.into(),
            message: msg.into(),
        });
    }

    /// Report a problem unless `v` is absent or one of `allowed`.
    pub fn one_of(&mut self, path: &str, v: Option<&str>, allowed: &[&str]) {
        if let Some(s) = v {
            if !allowed.contains(&s) {
                self.bad(path, format!("{} not in {allowed:?}", quoted(s)));
            }
        }
    }

    /// All problems found; empty = valid.
    pub fn into_issues(self) -> Vec<Issue> {
        self.0
    }
}

/// `Err(Error::Invalid)` listing `issues`, unless there are none.
pub fn into_result(issues: Vec<Issue>) -> Result<()> {
    if issues.is_empty() {
        Ok(())
    } else {
        Err(Error::Invalid(issues))
    }
}
