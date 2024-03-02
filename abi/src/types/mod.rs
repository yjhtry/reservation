mod request;
mod reservation;
mod reservation_query;
mod reservation_status;

use std::ops::Bound;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
pub use reservation::*;
use sqlx::postgres::types::PgRange;

use crate::{convert_to_utc_time, Error};

pub fn validate_range(start: Option<&Timestamp>, end: Option<&Timestamp>) -> Result<(), Error> {
    if start.is_none() || end.is_none() {
        return Err(Error::InvalidTime);
    }

    let start = start.unwrap().seconds;
    let end = end.unwrap().seconds;

    if start > end {
        return Err(Error::InvalidTime);
    }

    Ok(())
}

pub fn get_timespan(start: Option<&Timestamp>, end: Option<&Timestamp>) -> PgRange<DateTime<Utc>> {
    let start = convert_to_utc_time(start.unwrap());
    let end = convert_to_utc_time(end.unwrap());

    PgRange {
        start: Bound::Included(start),
        end: Bound::Excluded(end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_range_should_resolve_correct_range() {
        let start = Some(Timestamp {
            seconds: 0,
            nanos: 0,
        });
        let end = Some(Timestamp {
            seconds: 1,
            nanos: 0,
        });

        assert_eq!(validate_range(start.as_ref(), end.as_ref()), Ok(()));
    }

    #[test]
    fn validate_range_should_reject_wrong_range() {
        let start = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 0,
            nanos: 0,
        };

        assert_eq!(
            validate_range(Some(&start), Some(&end)),
            Err(Error::InvalidTime)
        );
    }

    #[test]
    fn get_timespan_should_work_for_valid_start_end() {
        let start = Timestamp {
            seconds: 0,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 1,
            nanos: 0,
        };

        let timespan = get_timespan(Some(&start), Some(&end));

        assert_eq!(timespan.start, Bound::Included(convert_to_utc_time(&start)));
        assert_eq!(timespan.end, Bound::Excluded(convert_to_utc_time(&end)));
    }
}
