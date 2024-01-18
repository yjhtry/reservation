use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use prost_types::Timestamp;

pub fn convert_to_utc_time(ts: Timestamp) -> Option<DateTime<Utc>> {
    let naive_datetime = NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32);
    naive_datetime.map(|ndt| Utc.from_utc_datetime(&ndt))
}

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}
