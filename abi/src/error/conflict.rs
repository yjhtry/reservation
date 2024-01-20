#![allow(clippy::all, dead_code)]

use std::{convert::Infallible, str::FromStr};

use chrono::{DateTime, Utc};

/// "Key (resource_id, timespan)=(resource_id, [\"2024-01-02 07:00:00+00\",
/// \"2024-01-04 07:00:00+00\")) conflicts with existing key (resource_id, timespan)
/// =(resource_id, [\"2024-01-01 07:00:00+00\",\"2024-01-03 07:00:00+00\"))

#[derive(Debug, Clone)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug, Clone, Default)]
pub struct ReservationConflict {
    a: ReservationWindow,
    b: ReservationWindow,
}

#[derive(Debug, Clone, Default)]
pub struct ReservationWindow {
    rid: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ReservationConflictInfo::Unparsed(s.to_string()))
    }
}
