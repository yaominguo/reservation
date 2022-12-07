use crate::{ReservationId, Rsvp};
use async_trait::async_trait;
use sqlx::{types::Uuid, PgPool, Row};

#[derive(Debug)]
pub struct ReservationManager {
    pool: PgPool,
}

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        rsvp.validate()?;

        let status = abi::ReservationStatus::from_i32(rsvp.status) // 数字转枚举值
            .unwrap_or(abi::ReservationStatus::Pending);
        let timespan = rsvp.get_timespan();
        let id:Uuid = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, resource_id, timespan, note, status) VALUES ($1, $2, $3, $4, $5::rsvp.reservation_status) RETURNING id"
        )
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?.get(0);

        rsvp.id = id.to_string();

        Ok(rsvp)
    }

    async fn change_status(&self, _id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn delete(&self, _id: ReservationId) -> Result<(), abi::Error> {
        todo!()
    }

    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        todo!()
    }

    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        todo!()
    }
}

impl ReservationManager {
    #[allow(dead_code)]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = abi::Reservation::new_pending(
            "user_id",
            "resource_id",
            "2022-12-25T12:00:00-0700".parse().unwrap(),
            "2022-12-31T12:00:00-0700".parse().unwrap(),
            "Test note",
        );
        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }
}
