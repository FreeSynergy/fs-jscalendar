//! Rules for the members every JSCalendar object has.
//!
//! Pattern: Specification — one function checks the common members through a JSON view, so it
//! serves `Event` and `Task`; the recurrence rule has its own file.

use serde_json::Value;

use super::ctx::Ctx;
use super::rule::rule;
use crate::error::{quoted, short};
use crate::model::{PARTICIPATION_STATUSES, RecurrenceRule, VERSION};
use crate::time::{Zone, parse_local, parse_utc};

/// `@type`, `version` and `uid` of an object whose `@type` must be `want`.
pub fn common(c: &mut Ctx, ty: Option<&str>, want: &str, version: Option<&str>, uid: &str) {
    if ty != Some(want) {
        c.bad("/@type", format!("must be {want:?}"));
    }
    if version != Some(VERSION) {
        c.bad(
            "/version",
            "required, must be \"2.0\" (upgrade RFC 8984 data with `upgrade`)",
        );
    }
    if uid.trim().is_empty() {
        c.bad("/uid", "required");
    }
}

/// Fields every object has, read through a JSON view so one function serves all of them.
pub fn shared<T: serde::Serialize>(c: &mut Ctx, obj: &T) {
    let v = serde_json::to_value(obj).unwrap_or(Value::Null);
    let s = |k: &str| v.get(k).and_then(Value::as_str);
    if let Some(st) = s("start") {
        if parse_local(st).is_err() {
            c.bad(
                "/start",
                format!(
                    "not a LocalDateTime (no zone suffix, no fraction in 2.0): {}",
                    quoted(st)
                ),
            );
        }
    }
    for k in ["created", "updated"] {
        if let Some(t) = s(k) {
            if parse_utc(t).is_err() {
                c.bad(
                    &format!("/{k}"),
                    format!("not a UTCDateTime: {}", quoted(t)),
                );
            }
        }
    }
    for k in ["timeZone", "recurrenceIdTimeZone"] {
        if let Some(z) = s(k) {
            if Zone::parse(z).is_err() {
                c.bad(
                    &format!("/{k}"),
                    format!(
                        "not an IANA time zone (JSCalendar 2.0 has no custom zones): {}",
                        quoted(z)
                    ),
                );
            }
        }
    }
    if v.get("recurrenceRules").is_some() {
        c.bad(
            "/recurrenceRules",
            "obsolete in 2.0, use the single `recurrenceRule`",
        );
    }
    if v.get("timeZones").is_some() || v.get("excludedRecurrenceRules").is_some() {
        c.bad(
            "/timeZones",
            "`timeZones` and `excludedRecurrenceRules` are obsolete in 2.0",
        );
    }
    c.one_of("/freeBusyStatus", s("freeBusyStatus"), &["free", "busy"]);
    c.one_of("/privacy", s("privacy"), &["public", "private", "secret"]);
    if let Some(r) = v
        .get("recurrenceRule")
        .and_then(|r| serde_json::from_value::<RecurrenceRule>(r.clone()).ok())
    {
        rule(c, &r);
        if s("start").is_none() {
            c.bad("/recurrenceRule", "needs `start`");
        }
    }
    if let Some(ov) = v.get("recurrenceOverrides").and_then(Value::as_object) {
        for k in ov.keys() {
            if parse_local(k).is_err() {
                c.bad(
                    &format!("/recurrenceOverrides/{}", short(k)),
                    "key must be a LocalDateTime",
                );
            }
        }
    }
    if let Some(ps) = v.get("participants").and_then(Value::as_object) {
        for (k, p) in ps {
            if let Some(st) = p.get("participationStatus").and_then(Value::as_str) {
                if !PARTICIPATION_STATUSES.contains(&st) {
                    c.bad(
                        &format!("/participants/{}/participationStatus", short(k)),
                        format!("{} not in {PARTICIPATION_STATUSES:?}", quoted(st)),
                    );
                }
            }
            for obsolete in ["language", "locationId"] {
                if p.get(obsolete).is_some() {
                    c.bad(
                        &format!("/participants/{}/{obsolete}", short(k)),
                        "obsolete in 2.0",
                    );
                }
            }
        }
    }
}
