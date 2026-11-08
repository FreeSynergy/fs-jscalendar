//! What used to pass or get lost: fractions after an upgrade, folded lines, second 60, huge values in messages.

use fs_jscalendar::ical::{Component, split_ics, unfolded_lines};
use fs_jscalendar::quoted;
use fs_jscalendar::time::{Dur, FOREVER, Zone, parse_local, parse_utc};
use fs_jscalendar::upgrade::upgrade;
use fs_jscalendar::validate::{Ctx, shared};
use serde_json::json;

fn all_issues(v: &serde_json::Value) -> Vec<String> {
    let mut c = Ctx::new();
    shared(&mut c, v);
    c.into_issues().iter().map(ToString::to_string).collect()
}

#[test]
fn rfc_8984_fractions_validate_after_upgrade_and_their_originals_are_kept() {
    let old = json!({
        "@type": "Task", "uid": "u",
        "created": "2026-01-01T10:00:00.25Z",
        "updated": "2026-01-02T10:00:00.5Z",
        "progressUpdated": "2026-01-03T10:00:00.125Z",
        "start": "2026-01-04T10:00:00.5",
        "due": "2026-01-05T10:00:00.5Z",
        "estimatedDuration": "PT1.5S",
        "recurrenceRule": {"frequency": "daily", "until": "2026-02-01T10:00:00.5"},
        "recurrenceOverrides": {
            "2026-01-05T10:00:00.5": {"title": "a", "updated": "2026-01-02T10:00:00.5Z"},
            "2026-01-05T10:00:00.7": {"title": "b"},
        },
    });
    assert!(!all_issues(&old).is_empty(), "these are invalid before");
    let up = upgrade(old).value;
    assert_eq!(all_issues(&up), Vec::<String>::new());
    assert_eq!(up["created"], "2026-01-01T10:00:00Z");
    assert_eq!(up["updated"], "2026-01-02T10:00:00Z");
    assert_eq!(up["progressUpdated"], "2026-01-03T10:00:00Z");
    assert_eq!(up["estimatedDuration"], "PT2S");
    assert_eq!(up["recurrenceRule"]["until"], "2026-02-01T10:00:00");
    let patch = &up["recurrenceOverrides"]["2026-01-05T10:00:00"];
    assert_eq!(patch["updated"], "2026-01-02T10:00:00Z");
    let legacy = &up["freesynergy.net:legacy"];
    assert_eq!(legacy["updatedOriginal"], "2026-01-02T10:00:00.5Z");
    assert_eq!(legacy["estimatedDurationOriginal"], "PT1.5S");
    assert_eq!(
        legacy["recurrenceRule/untilOriginal"],
        "2026-02-01T10:00:00.5"
    );
    assert_eq!(legacy["dueOriginal"], "2026-01-05T10:00:00.5Z");
    assert_eq!(
        legacy["recurrenceOverrides/2026-01-05T10:00:00.7"]["title"],
        "b"
    );
    assert_eq!(upgrade(up.clone()).value, up, "idempotent");
}

#[test]
fn durations_round_up_and_what_is_no_fraction_stays() {
    for (old, new) in [
        ("PT1.5S", "PT2S"),
        ("PT0.1S", "PT1S"),
        ("PT1.0S", "PT1S"),
        ("P1DT2H3M4.5S", "P1DT2H3M5S"),
        ("PT59.9S", "PT60S"),
        ("PT1S", "PT1S"),
        ("PT1.5H", "PT1.5H"),
        ("PT18446744073709551615.5S", "PT18446744073709551615.5S"),
    ] {
        let up = upgrade(json!({"@type": "Task", "uid": "u", "estimatedDuration": old}));
        assert_eq!(up.value["estimatedDuration"], new, "{old}");
    }
    let garbage = upgrade(json!({"@type": "Task", "uid": "u", "updated": "not.a.time"}));
    assert_eq!(
        garbage.value["updated"], "not.a.time",
        "left for the validator to name"
    );
    assert_ne!(all_issues(&garbage.value), [] as [std::string::String; 0]);
}

const TODO_A: &str = "BEGIN:VTODO\r\nUID:a\r\nDTSTAMP:20261101T000000Z\r\nDESCRIPTION:one\r\n END:VTODO\r\n still the description\r\nSUMMARY:kept\r\nEND:VTODO\r\n";
const TODO_B: &str = "BEGIN:VTODO\r\nUID:b\r\nDTSTAMP:20261101T000000Z\r\nDESCRIPTION:two\r\n BEGIN:VTODO\r\nSUMMARY:kept too\r\nEND:VT\r\n ODO\r\n";

#[test]
fn a_continuation_line_never_ends_or_begins_a_component() {
    let ics = format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nX-NOTE:\r\n END:VCALENDAR\r\n{TODO_A}{TODO_B}END:VCALENDAR\r\n"
    );
    let parts = split_ics(&ics, Component::Task);
    assert_eq!(parts.len(), 2, "{parts:?}");
    assert_eq!(parts[0].uid, "a");
    assert!(
        parts[0].text.contains(TODO_A),
        "the whole component, byte for byte"
    );
    assert!(
        parts[0].text.contains(" END:VCALENDAR\r\n"),
        "the header too"
    );
    assert!(!parts[0].text.contains("UID:b"));
    assert_eq!(parts[1].uid, "b");
    assert!(parts[1].text.contains(TODO_B), "a folded END still ends it");
}

#[test]
fn a_folded_uid_is_read_and_lines_come_unfolded() {
    let ics = "BEGIN:VCALENDAR\nBEGIN:VTODO\nUI\n D:long\n\t-uid\nEND:VTODO\nEND:VCALENDAR\n";
    assert_eq!(split_ics(ics, Component::Task)[0].uid, "long-uid");
    let lines = unfolded_lines("A:1\r\n 2\r\n\t3\r\nB:4  \r\n");
    assert_eq!(lines, ["A:123", "B:4"]);
}

#[test]
fn second_sixty_is_refused_and_the_far_end_still_parses() {
    assert!(parse_local("2026-11-24T10:15:60").is_err());
    assert!(
        parse_local("2016-12-31T23:59:60").is_err(),
        "a real leap second too"
    );
    assert!(parse_utc("2026-11-24T10:15:60Z").is_err());
    assert!(parse_local("2026-11-24T10:60:00").is_err());
    assert!(parse_local("2026-11-24T10:15:59").is_ok());
    assert!(parse_utc(FOREVER).is_ok());
}

#[test]
fn a_message_quotes_a_value_short_and_escaped() {
    let huge = "\u{7f}".repeat(1024 * 1024);
    for e in [
        parse_local(&huge).unwrap_err(),
        parse_utc(&huge).unwrap_err(),
        Dur::parse(&huge).unwrap_err(),
        Zone::parse(&huge).unwrap_err(),
        Zone::parse(&format!("/{huge}")).unwrap_err(),
    ] {
        assert!(e.to_string().len() < 1024, "{}", e.to_string().len());
    }
    let v = json!({
        "start": huge, "updated": huge, "timeZone": huge, "privacy": huge,
        "recurrenceRule": {"frequency": huge, "until": huge, "byDay": [{"day": huge}]},
        "recurrenceOverrides": {huge.clone(): {}},
        "participants": {huge.clone(): {"participationStatus": huge, "language": "de"}},
    });
    let issues = all_issues(&v);
    assert!(issues.len() >= 9, "{}", issues.len());
    for issue in issues {
        assert!(issue.len() < 1024, "{}", issue.len());
    }
    assert_eq!(quoted("short"), "\"short\"");
    assert!(quoted(&"é".repeat(100)).ends_with("... (200 bytes)"));
}
