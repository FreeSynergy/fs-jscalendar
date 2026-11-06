//! Whole seconds for the upgrade: JSCalendar 2.0 has no fractions in date-times or durations.
//!
//! Pattern: Pipeline step (of `upgrade`) — one family of values, every original kept in the legacy
//! property. Date-times are cut, as the seconds are what they name. Durations round up, so a duration
//! that was not zero does not become zero and nothing ends before it did.

use serde_json::{Map, Value, json};

use crate::model::LEGACY_KEY;

/// Date-times of an object, with or without their `Z`.
const TIMES: [&str; 4] = ["updated", "created", "progressUpdated", "recurrenceId"];
/// Durations of an object: a task's and an event's.
const DURATIONS: [&str; 2] = ["estimatedDuration", "duration"];

/// `s` without its fraction, its `Z` kept; `None` if it has no fraction of digits.
pub(crate) fn whole_seconds(s: &str) -> Option<String> {
    let (whole, rest) = s.split_once('.')?;
    let (digits, suffix) = match rest.strip_suffix('Z') {
        Some(digits) => (digits, "Z"),
        None => (rest, ""),
    };
    all_digits(digits).then(|| format!("{whole}{suffix}"))
}

/// `PT1.5S` as `PT2S`; `None` if there is no fraction of seconds, or the number is out of range.
fn whole_duration(s: &str) -> Option<String> {
    let (head, rest) = s.split_once('.')?;
    let digits = rest.strip_suffix('S').filter(|d| all_digits(d))?;
    let start = head.len() - head.bytes().rev().take_while(u8::is_ascii_digit).count();
    let seconds: u64 = head[start..].parse().ok()?;
    let up = u64::from(digits.bytes().any(|b| b != b'0'));
    Some(format!("{}{}S", &head[..start], seconds.checked_add(up)?))
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

struct Round<'a> {
    legacy: &'a mut Map<String, Value>,
    notes: &'a mut Vec<String>,
    at: &'a str,
}

impl Round<'_> {
    /// Rewrite member `key` of `o`, keeping the original under `name`.
    fn member(
        &mut self,
        o: &mut Map<String, Value>,
        key: &str,
        name: &str,
        rewrite: impl Fn(&str) -> Option<String>,
    ) {
        let found = o.get(key).and_then(Value::as_str);
        let Some((original, whole)) = found.and_then(|s| Some((s.to_string(), rewrite(s)?))) else {
            return;
        };
        o.insert(key.into(), json!(whole));
        self.legacy
            .insert(format!("{name}Original"), json!(original));
        self.notes.push(format!(
            "{}: fractional seconds removed from `{name}`, original in {LEGACY_KEY}",
            self.at
        ));
    }

    fn all(&mut self, o: &mut Map<String, Value>, prefix: &str) {
        for key in TIMES {
            self.member(o, key, &format!("{prefix}{key}"), whole_seconds);
        }
        for key in DURATIONS {
            self.member(o, key, &format!("{prefix}{key}"), whole_duration);
        }
    }
}

/// Whole seconds in the date-times and durations of `o`, in its recurrence rule and in its overrides.
pub(crate) fn round_times(
    o: &mut Map<String, Value>,
    legacy: &mut Map<String, Value>,
    notes: &mut Vec<String>,
    at: &str,
) {
    let mut round = Round { legacy, notes, at };
    round.all(o, "");
    if let Some(Value::Object(rule)) = o.get_mut("recurrenceRule") {
        round.member(rule, "until", "recurrenceRule/until", whole_seconds);
    }
    if let Some(Value::Object(overrides)) = o.get_mut("recurrenceOverrides") {
        for (key, patch) in overrides.iter_mut() {
            if let Value::Object(patch) = patch {
                round.all(patch, &format!("recurrenceOverrides/{key}/"));
                for member in ["start", "due"] {
                    let name = format!("recurrenceOverrides/{key}/{member}");
                    round.member(patch, member, &name, whole_seconds);
                }
            }
        }
    }
}
