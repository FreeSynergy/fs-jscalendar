//! Durations from outside cannot break date arithmetic.

use chrono::NaiveDate;
use fs_jscalendar::time::{Dur, Zone, parse_local, parse_utc};

#[test]
fn an_absurd_duration_is_refused_not_a_panic_later() {
    for s in [
        "P99999999999W",
        "P9223372036854775807D",
        "PT9223372036854775807H",
        "P3650001D",
    ] {
        assert!(Dur::parse(s).is_err(), "{s}");
    }
}

#[test]
fn the_longest_accepted_duration_can_be_calculated_with() {
    let longest = Dur::parse("P3650000D").unwrap();
    let start = NaiveDate::from_ymd_opt(2026, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    assert!(longest.end_utc(&Zone::Utc, start).timestamp() > 0);
    assert!(longest.max_secs() > 0);
}

#[test]
fn the_longest_duration_from_the_latest_start_stays_in_range_in_any_zone() {
    let longest = Dur::parse("P3650000DT87600000H").unwrap();
    let last = parse_local("9999-12-31T23:59:59").unwrap();
    for zone in [
        "Utc",
        "Europe/Berlin",
        "Pacific/Kiritimati",
        "Pacific/Pago_Pago",
    ] {
        let zone = if zone == "Utc" {
            Zone::Utc
        } else {
            Zone::parse(zone).unwrap()
        };
        assert!(longest.end_utc(&zone, last).timestamp() > 0);
    }
}

#[test]
fn only_the_plain_form_is_a_date_time() {
    for s in [
        "+262142-1-1T00:00:0",
        "+67562-01-1T00:00:0",
        "2026-1-01T00:00:00 ",
        "2026-01-01 00:00:00",
    ] {
        assert!(parse_local(s).is_err(), "{s}");
        assert!(parse_utc(&format!("{s}Z")).is_err(), "{s}");
    }
    assert!(parse_local("2026-11-24T10:00:00").is_ok());
    assert!(parse_utc("2026-11-24T10:00:00Z").is_ok());
}
