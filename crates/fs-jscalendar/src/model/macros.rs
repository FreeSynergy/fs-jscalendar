//! Macros that fix the skeleton of every JSCalendar object.
//!
//! Pattern: Template method (macro form) — [`typed_object!`](crate::typed_object) adds `@type` and
//! the flattened `extra` map (lossless round trips); [`calendar_object!`](crate::calendar_object)
//! adds the members and common methods of an object, so `Event` and `Task` only list what is
//! their own. The owner's crate needs `serde` for the derives.

/// A serde struct with `@type`, the listed members and a lossless `extra` map.
#[macro_export]
macro_rules! typed_object {
    ($(#[$m:meta])* $name:ident { $($body:tt)* }) => {
        $(#[$m])*
        #[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(default)]
        pub struct $name {
            #[serde(rename = "@type", skip_serializing_if = "Option::is_none")]
            pub type_: Option<String>,
            $($body)*
            #[serde(flatten)]
            pub extra: $crate::model::Extra,
        }
    };
}

/// A JSCalendar object: the members every object has, its own members, and the common methods.
#[macro_export]
macro_rules! calendar_object {
    ($(#[$m:meta])* $name:ident, $tag:literal { $($body:tt)* }) => {
        $crate::typed_object!(
            $(#[$m])*
            $name {
                #[serde(skip_serializing_if = "Option::is_none")]
                pub version: Option<String>,
                pub uid: String,
                #[serde(rename = "prodId", skip_serializing_if = "Option::is_none")]
                pub prod_id: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub created: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub updated: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub sequence: Option<u32>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub title: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub description: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub locale: Option<String>,
                /// `LocalDateTime`, interpreted in `timeZone` (floating if absent).
                #[serde(skip_serializing_if = "Option::is_none")]
                pub start: Option<String>,
                #[serde(rename = "timeZone", skip_serializing_if = "Option::is_none")]
                pub time_zone: Option<String>,
                #[serde(rename = "showWithoutTime", skip_serializing_if = "Option::is_none")]
                pub show_without_time: Option<bool>,
                #[serde(rename = "freeBusyStatus", skip_serializing_if = "Option::is_none")]
                pub free_busy_status: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub privacy: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                pub priority: Option<i32>,
                #[serde(rename = "recurrenceId", skip_serializing_if = "Option::is_none")]
                pub recurrence_id: Option<String>,
                #[serde(rename = "recurrenceIdTimeZone", skip_serializing_if = "Option::is_none")]
                pub recurrence_id_time_zone: Option<String>,
                #[serde(rename = "recurrenceRule", skip_serializing_if = "Option::is_none")]
                pub recurrence_rule: Option<$crate::model::RecurrenceRule>,
                /// Key: `LocalDateTime` of the instance; value: patch object (`{"excluded":true}`, `{"title":"..."}`, `{"participants/p1/participationStatus":"declined"}`).
                #[serde(rename = "recurrenceOverrides", skip_serializing_if = "std::collections::BTreeMap::is_empty")]
                pub recurrence_overrides: $crate::model::Map<$crate::__json::Value>,
                #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
                pub participants: $crate::model::Map<$crate::model::Participant>,
                /// RFC 9253 relations: key = `uid` of the other object, value = `Relation`.
                #[serde(rename = "relatedTo", skip_serializing_if = "std::collections::BTreeMap::is_empty")]
                pub related_to: $crate::model::Map<$crate::__json::Value>,
                /// Labels.
                #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
                pub keywords: $crate::model::Map<bool>,
                $($body)*
            }
        );
        impl $name {
            /// The JSCalendar `@type` of this object.
            pub const TYPE: &'static str = $tag;
            /// Set `@type` and `version` if missing.
            pub fn normalise(&mut self) {
                self.type_.get_or_insert_with(|| $tag.to_string());
                self.version.get_or_insert_with(|| $crate::model::VERSION.to_string());
            }
            /// Vendor / unknown property.
            pub fn vendor(&self, key: &str) -> Option<&$crate::__json::Value> {
                self.extra.get(key)
            }
            /// Set a vendor / unknown property.
            pub fn set_vendor(&mut self, key: &str, v: $crate::__json::Value) {
                self.extra.insert(key.to_string(), v);
            }
            /// `title`, or a placeholder (the envelope `name` may never be empty, ADR 0032).
            pub fn display_title(&self) -> String {
                match self.title.as_deref().map(str::trim) {
                    Some(t) if !t.is_empty() => t.to_string(),
                    _ => format!("(untitled {})", $tag.to_lowercase()),
                }
            }
            /// The object as a JSON value (the stored content).
            ///
            /// # Panics
            /// Never in practice: the object holds only JSON-compatible data.
            pub fn to_value(&self) -> $crate::__json::Value {
                $crate::__json::to_value(self).expect("serialises")
            }
            /// Read an object from JSON.
            pub fn from_value(v: $crate::__json::Value) -> $crate::Result<Self> {
                Ok($crate::__json::from_value(v)?)
            }
            /// Key of the participant with this e-mail address (case-insensitive).
            pub fn participant_by_email(&self, email: &str) -> Option<&str> {
                let want = email.trim().trim_start_matches("mailto:").to_lowercase();
                self.participants.iter().find_map(|(k, p)| {
                    let e = p.email.as_deref().or(p.calendar_address.as_deref())?;
                    (e.trim().trim_start_matches("mailto:").to_lowercase() == want).then_some(k.as_str())
                })
            }
        }
    };
}
