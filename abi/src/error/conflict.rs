use std::{convert::Infallible, str::FromStr};

use chrono::{DateTime, Utc};
use prost_types::Timestamp;

use crate::convert_to_utc_time;

#[derive(Debug, Clone)]
pub enum ReservationConflictInfo {
    Parsed(ReservationWindow),
    Unparsed(String),
}

#[derive(Debug, Clone, Default)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl ReservationWindow {
    pub fn new(rid: String, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { rid, start, end }
    }
}

impl FromStr for ReservationWindow {
    type Err = String;

    /// "Key (resource_id, timespan)=(resource_id, [\"2024-01-02 07:00:00+00\",
    /// \"2024-01-04 07:00:00+00\")) conflicts with existing key (resource_id_a, timespan)
    /// =(resource_id_b, [\"2024-01-01 07:00:00+00\",\"2024-01-03 07:00:00+00\"))

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r"(?:=\((?<old>[^\(\(]*)\){2})\.?$").unwrap();
        if let Some(caps) = re.captures(s) {
            let split = caps["old"].splitn(2, ',').collect::<Vec<_>>();

            let rid = split[0].trim().to_string();
            let timespan = split[1].trim().to_string().replace(['[', '"'], "");

            let timespan = timespan.splitn(2, ',').collect::<Vec<_>>();
            let start = timespan[0].trim().to_string();
            let end = timespan[1].trim().to_string();

            Ok(ReservationWindow::new(
                rid,
                convert_to_utc_time(Timestamp::from_str(&start).unwrap()).unwrap(),
                convert_to_utc_time(Timestamp::from_str(&end).unwrap()).unwrap(),
            ))
        } else {
            Err(s.to_string())
        }
    }
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let window = ReservationWindow::from_str(s);
        if let Ok(window) = window {
            return Ok(ReservationConflictInfo::Parsed(window));
        }

        Ok(ReservationConflictInfo::Unparsed(s.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let s = r#"Key (resource_id, timespan)=(resource_id, ["2024-01-02 07:00:00+00", "2024-01-04 07:00:00+00")) conflicts with existing key (resource_id, timespan)=(resource_id, ["2024-01-01 07:00:00+00","2024-01-03 07:00:00+00"))"#;
        let info = s.parse::<ReservationConflictInfo>().unwrap();
        assert!(matches!(info, ReservationConflictInfo::Parsed(_)));
    }
}
