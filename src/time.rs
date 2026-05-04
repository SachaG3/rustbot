use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{Europe::Paris, Tz};

pub fn paris_now() -> DateTime<Tz> {
    Utc::now().with_timezone(&Paris)
}

pub fn paris_today() -> NaiveDate {
    paris_now().date_naive()
}

pub fn paris_now_naive() -> NaiveDateTime {
    paris_now().naive_local()
}

pub fn paris_date_days_ago(days_ago: i64) -> NaiveDate {
    paris_today() - chrono::Duration::days(days_ago)
}

pub fn paris_day_bounds_utc(days_ago: i64) -> (DateTime<Utc>, DateTime<Utc>) {
    let date = paris_date_days_ago(days_ago);
    let start = date.and_hms_opt(0, 0, 0).unwrap();
    let end = date.and_hms_opt(23, 59, 59).unwrap();

    let start_paris = Paris.from_local_datetime(&start).earliest().unwrap();
    let end_paris = Paris.from_local_datetime(&end).latest().unwrap();

    (
        start_paris.with_timezone(&Utc),
        end_paris.with_timezone(&Utc),
    )
}
