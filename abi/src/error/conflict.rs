use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

#[derive(Debug)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub exist: ReservationWindow,
}

#[derive(Debug)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

// help str.parse() convert to ReservationConflictInfo succeed
impl FromStr for ReservationConflictInfo {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // if s can be converted to ReservationConflict
        if let Ok(conflict) = s.parse() {
            Ok(ReservationConflictInfo::Parsed(conflict))
        } else {
            Ok(ReservationConflictInfo::Unparsed(s.to_string()))
        }
    }
}

// help str.parse() convert to ReservationConflict succeed
impl FromStr for ReservationConflict {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<ParsedInfo>()?.try_into()
    }
}

// help ParsedInfo.try_into() convert to ReservationConflict succeed
impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();
    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            exist: value.exist.try_into()?,
        })
    }
}

// help HashMap.try_into convert to ReservationWindow succeed
impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();
    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");
        let mut split = timespan_str.splitn(2, ',');
        let start = parse_datetime(split.next().ok_or(())?)?;
        let end = parse_datetime(split.next().ok_or(())?)?;
        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

struct ParsedInfo {
    new: HashMap<String, String>,
    exist: HashMap<String, String>,
}

// help str.parse::<ParsedInfo>() convert to ParsedInfo succeed
impl FromStr for ParsedInfo {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = Regex::new(r#"\((?P<k1>[a-zA-Z0-9_-]+)\s*,\s*(?P<k2>[a-zA-Z0-9_-]+)\)=\((?P<v1>[a-zA-Z0-9_-]+)\s*,\s*\[(?P<v2>[^\)\]]+)"#).unwrap();
        let mut maps = vec![];
        for cap in r.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(cap["k1"].to_string(), cap["v1"].to_string());
            map.insert(cap["k2"].to_string(), cap["v2"].to_string());
            maps.push(Some(map));
        }
        if maps.len() != 2 {
            return Err(());
        }
        Ok(ParsedInfo {
            new: maps[0].take().unwrap(),
            exist: maps[1].take().unwrap(),
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .with_timezone(&Utc))
}

#[cfg(test)]

mod tests {
    use super::*;
    const ERR_MSG:&str =  "Key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\")).";
    #[test]
    fn parsed_info_should_work() {
        let info: ParsedInfo = ERR_MSG.parse().unwrap();
        assert_eq!(info.new["resource_id"], "ocean-view-room-713");
        assert_eq!(
            info.new["timespan"],
            "\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\""
        );
        assert_eq!(info.exist["resource_id"], "ocean-view-room-713");
        assert_eq!(
            info.exist["timespan"],
            "\"2022-12-25 22:00:00+00\",\"2022-12-28 19:00:00+00\""
        );
    }

    #[test]
    fn hashmap_to_reservationwindow_should_work() {
        let mut map = HashMap::new();
        map.insert("resource_id".to_string(), "ocean-view-room-713".to_string());
        map.insert(
            "timespan".to_string(),
            "\"2022-12-26 22:00:00+00\",\"2022-12-30 19:00:00+00\"".to_string(),
        );
        let window: ReservationWindow = map.try_into().unwrap();
        assert_eq!(window.rid, "ocean-view-room-713");
        assert_eq!(window.start.to_rfc3339(), "2022-12-26T22:00:00+00:00");
        assert_eq!(window.end.to_rfc3339(), "2022-12-30T19:00:00+00:00");
    }

    #[test]
    fn conflict_error_message_should_parse() {
        let info: ReservationConflictInfo = ERR_MSG.parse().unwrap();

        match info {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.new.rid, "ocean-view-room-713");
                assert_eq!(conflict.new.start.to_rfc3339(), "2022-12-26T22:00:00+00:00");
                assert_eq!(conflict.new.end.to_rfc3339(), "2022-12-30T19:00:00+00:00");
                assert_eq!(conflict.exist.rid, "ocean-view-room-713");
                assert_eq!(
                    conflict.exist.start.to_rfc3339(),
                    "2022-12-25T22:00:00+00:00"
                );
                assert_eq!(conflict.exist.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
            }
            ReservationConflictInfo::Unparsed(_) => panic!("should be parsed!"),
        }
    }
}
