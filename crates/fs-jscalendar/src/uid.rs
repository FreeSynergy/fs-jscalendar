//! Fresh `uid` values for JSCalendar objects.
//!
//! Pattern: none (pure helper) — one place for the `urn:uuid:` form, UUIDv7 sorts by creation time.

/// A new `uid`: `urn:uuid:` followed by a UUIDv7.
pub fn new_uid() -> String {
    format!("urn:uuid:{}", uuid::Uuid::now_v7())
}
