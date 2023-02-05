use std::ops::Bound;

use crate::{
    convert_to_timestamp, convert_to_utc_time, Error, Reservation, ReservationStatus, RsvpStatus,
};
use chrono::{DateTime, FixedOffset, Utc};
use sqlx::{
    postgres::{types::PgRange, PgRow},
    types::Uuid,
    FromRow, Row,
};

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id: uid.into(),
            status: ReservationStatus::Pending as i32,
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(end.with_timezone(&Utc))),
            note: note.into(),
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        if self.start.is_none() || self.end.is_none() {
            return Err(Error::InvalidTime);
        }

        let start = convert_to_utc_time(self.start.as_ref().unwrap().clone());
        let end = convert_to_utc_time(self.end.as_ref().unwrap().clone());

        if start >= end {
            return Err(Error::InvalidTime);
        }

        Ok(())
    }

    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        let start = convert_to_utc_time(self.start.as_ref().unwrap().clone());
        let end = convert_to_utc_time(self.end.as_ref().unwrap().clone());
        (start..end).into()
    }
}

impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id: Uuid = row.get("id");
        let range: NaiveRange<DateTime<Utc>> =
            row.get::<PgRange<DateTime<Utc>>, &str>("timespan").into();
        let status: RsvpStatus = row.get("status");

        Ok(Self {
            id: id.to_string(),
            user_id: row.get("user_id"),
            status: ReservationStatus::from(status) as i32,
            resource_id: row.get("resource_id"),
            start: Some(convert_to_timestamp(range.start.unwrap())),
            end: Some(convert_to_timestamp(range.end.unwrap())),
            note: row.get("note"),
        })
    }
}

struct NaiveRange<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T> From<PgRange<T>> for NaiveRange<T> {
    fn from(range: PgRange<T>) -> Self {
        let get_from = |date: Bound<T>| match date {
            Bound::Included(v) => Some(v),
            Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        };
        Self {
            start: get_from(range.start),
            end: get_from(range.end),
        }
    }
}
