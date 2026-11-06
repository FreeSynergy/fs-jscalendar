//! Durations from outside cannot break date arithmetic.

use chrono::NaiveDate;
use fs_jscalendar::time::{Dur, Zone};

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
