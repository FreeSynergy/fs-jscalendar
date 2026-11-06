//! The shared parts on their own, without an owner's object.

use fs_jscalendar::ical::{Component, parse_entries, split_ics, write_ics};
use fs_jscalendar::model::{Participant, next_participant_key};
use fs_jscalendar::time::{Dur, Zone, parse_local, parse_utc};
use fs_jscalendar::upgrade::upgrade;
use fs_jscalendar::validate::{Ctx, common, into_result, shared};
use fs_jscalendar::{Error, new_uid, now_utc};
use serde_json::json;

const MIXED: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:x\r\nBEGIN:VEVENT\r\nUID:e1\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20260105T090000Z\r\nSUMMARY:E\r\nEND:VEVENT\r\nBEGIN:VTODO\r\nUID:t1\r\nDTSTAMP:20260101T000000Z\r\nSUMMARY:T\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";

#[test]
fn split_keeps_each_kind_with_its_owner() {
    let events = split_ics(MIXED, Component::Event);
    let tasks = split_ics(MIXED, Component::Task);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].uid, "e1");
    assert!(!events[0].text.contains("VTODO"));
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].uid, "t1");
    assert!(!tasks[0].text.contains("VEVENT"));
}

#[test]
fn entries_are_filtered_by_type_and_written_back() {
    let (tasks, _) = parse_entries(MIXED, "Task").unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["title"], "T");
    let text = write_ics(tasks, "-//test//EN").unwrap();
    assert!(text.contains("BEGIN:VTODO"));
    assert!(!text.contains("BEGIN:VEVENT"));
}

#[test]
fn time_values_parse_and_reject() {
    assert!(parse_local("2026-11-24T10:00:00").is_ok());
    assert!(matches!(
        parse_local("2026-11-24T10:00:00Z"),
        Err(Error::Time(_))
    ));
    assert!(parse_utc(&now_utc()).is_ok());
    assert_eq!(Dur::parse("P1DT1H").unwrap().secs, 3600);
    assert!(matches!(Zone::parse("/custom"), Err(Error::Time(_))));
}

#[test]
fn upgrade_moves_to_two_point_zero() {
    let up = upgrade(json!({"@type": "Task", "uid": "u", "start": "2026-01-01T10:00:00Z"}));
    assert_eq!(up.value["version"], "2.0");
    assert_eq!(up.value["timeZone"], "Etc/UTC");
}

#[test]
fn shared_rules_report_all_problems_at_once() {
    let mut c = Ctx::new();
    common(&mut c, Some("Task"), "Event", None, " ");
    shared(
        &mut c,
        &json!({"start": "nope", "privacy": "x", "participants": {"p": {"participationStatus": "x"}}}),
    );
    let issues = c.into_issues();
    let paths: Vec<_> = issues.iter().map(|i| i.path.as_str()).collect();
    for want in [
        "/@type",
        "/version",
        "/uid",
        "/start",
        "/privacy",
        "/participants/p/participationStatus",
    ] {
        assert!(paths.contains(&want), "{want} missing in {paths:?}");
    }
    assert!(matches!(into_result(issues), Err(Error::Invalid(_))));
    assert!(into_result(vec![]).is_ok());
}

#[test]
fn participant_keys_and_uids() {
    let mut map = std::collections::BTreeMap::new();
    assert_eq!(next_participant_key(&map), "p1");
    map.insert("p1".to_string(), Participant::person("A", "a@x", "owner"));
    assert_eq!(next_participant_key(&map), "p2");
    assert_eq!(map["p1"].calendar_address.as_deref(), Some("mailto:a@x"));
    assert!(new_uid().starts_with("urn:uuid:"));
}
