//! A JSCalendar `Participant`.
//!
//! Pattern: Value object — plain serde data; flows that use it (invitations, assignment) live with
//! the owners of `Event` and `Task`.

// The members mirror the JSCalendar property names one to one; documenting each would only
// repeat the specification.
#![allow(missing_docs)]

use std::collections::BTreeMap;

use super::consts::Map;

crate::typed_object!(
    /// One participant of an object.
    Participant {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub email: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub kind: Option<String>,
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub roles: BTreeMap<String, bool>,
        #[serde(rename = "participationStatus", skip_serializing_if = "Option::is_none")]
        pub participation_status: Option<String>,
        #[serde(rename = "participationComment", skip_serializing_if = "Option::is_none")]
        pub participation_comment: Option<String>,
        #[serde(rename = "expectReply", skip_serializing_if = "Option::is_none")]
        pub expect_reply: Option<bool>,
        #[serde(rename = "calendarAddress", skip_serializing_if = "Option::is_none")]
        pub calendar_address: Option<String>,
    }
);

/// The JSCalendar names of `participationStatus`.
pub const PARTICIPATION_STATUSES: [&str; 5] = [
    "needs-action",
    "accepted",
    "declined",
    "tentative",
    "delegated",
];

impl Participant {
    /// A person with an e-mail address and one role, who has not answered yet.
    pub fn person(name: &str, email: &str, role: &str) -> Self {
        Self {
            type_: Some("Participant".into()),
            name: Some(name.into()),
            email: Some(email.into()),
            calendar_address: Some(format!("mailto:{email}")),
            kind: Some("individual".into()),
            roles: BTreeMap::from([(role.to_string(), true)]),
            participation_status: Some("needs-action".into()),
            ..Default::default()
        }
    }
}

/// The first free key (`p1`, `p2`, ...) in a participants map.
///
/// # Panics
/// Never: among `n + 1` candidate keys at most `n` are taken.
pub fn next_participant_key(participants: &Map<Participant>) -> String {
    (1..=participants.len() + 1)
        .map(|i| format!("p{i}"))
        .find(|k| !participants.contains_key(k))
        .expect("one of n + 1 candidate keys is free")
}
