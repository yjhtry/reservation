use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use prost_types::Timestamp;

pub fn convert_to_utc_time(ts: &Timestamp) -> DateTime<Utc> {
    let naive_datetime = NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32);
    naive_datetime
        .map(|ndt| Utc.from_utc_datetime(&ndt))
        .unwrap()
}

pub fn convert_to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_utc_time_should_resolve_correct_time() {
        let ts = Timestamp {
            seconds: 0,
            nanos: 0,
        };

        assert_eq!(
            convert_to_utc_time(&ts),
            Utc.timestamp_opt(0, 0).single().unwrap()
        );
    }

    #[test]
    fn convert_to_timestamp_should_resolve_correct_time() {
        let dt = Utc.timestamp_opt(0, 0).single().unwrap();

        assert_eq!(
            convert_to_timestamp(dt),
            Timestamp {
                seconds: 0,
                nanos: 0
            }
        );
    }
}
