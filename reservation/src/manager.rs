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

    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp = sqlx::query_as(
            "UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1 AND status = 'pending' RETURNING *"
        ).bind(id).fetch_one(&self.pool).await?;
        Ok(rsvp)
    }

    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp =
            sqlx::query_as("UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(rsvp)
    }

    async fn delete(&self, id: ReservationId) -> Result<(), abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        sqlx::query("DELETE FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp = sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(rsvp)
    }

    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        let user_id = str_to_option(&query.user_id);
        let resource_id = str_to_option(&query.resource_id);
        let range = query.get_timespan();
        let status = abi::ReservationStatus::from_i32(query.status)
            .unwrap_or(abi::ReservationStatus::Pending);
        let rsvps = sqlx::query_as(
            "SELECT * FROM rsvp.query($1, $2, $3, $4::rsvp.reservation_status, $5, $6, $7)",
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(range)
        .bind(status.to_string())
        .bind(query.page)
        .bind(query.page_size)
        .bind(query.desc)
        .fetch_all(&self.pool)
        .await?;

        Ok(rsvps)
    }
}

impl ReservationManager {
    #[allow(dead_code)]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn str_to_option(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use abi::{Reservation, ReservationConflictInfo, ReservationQuery, ReservationStatus};

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let (rsvp, _) = make_reservation(migrated_pool.clone()).await;
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_conflict_reservation_should_reject() {
        let (_, manager) = make_reservation(migrated_pool.clone()).await;
        let rsvp2 = abi::Reservation::new_pending(
            "user_id2",
            "resource_id",
            "2022-12-26T12:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "Test note2",
        );
        let err = manager.reserve(rsvp2).await.unwrap_err();
        if let abi::Error::ConflictReservation(ReservationConflictInfo::Parsed(info)) = err {
            assert_eq!(info.exist.rid, "resource_id");
            assert_eq!(info.exist.start.to_rfc3339(), "2022-12-25T19:00:00+00:00");
            assert_eq!(info.exist.end.to_rfc3339(), "2022-12-31T19:00:00+00:00");
        } else {
            panic!("expect conflict reservation error");
        }
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_change_status_should_work() {
        let (rsvp, manager) = make_reservation(migrated_pool.clone()).await;
        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        assert_eq!(rsvp.status, abi::ReservationStatus::Confirmed as i32)
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let (rsvp, manager) = make_reservation(migrated_pool.clone()).await;
        let rsvp = manager
            .update_note(rsvp.id, "Updated Note!!!".to_owned())
            .await
            .unwrap();

        assert_eq!(rsvp.note, "Updated Note!!!")
    }
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn get_and_delete_reservation_should_work() {
        let (rsvp, manager) = make_reservation(migrated_pool.clone()).await;
        let data = manager.get(rsvp.id.clone()).await.unwrap();
        assert_eq!(rsvp, data);
        let result = manager.delete(rsvp.id.clone()).await;
        assert!(result.is_ok());
    }

    async fn make_reservation(pool: PgPool) -> (Reservation, ReservationManager) {
        make_basic_reservation(
            pool,
            "user_id1",
            "resource_id",
            "2022-12-25T12:00:00-0700",
            "2022-12-31T12:00:00-0700",
            "Test note1",
        )
        .await
    }
    async fn make_basic_reservation(
        pool: PgPool,
        uid: &str,
        rid: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> (Reservation, ReservationManager) {
        let manager = ReservationManager::new(pool.clone());
        let rsvp = abi::Reservation::new_pending(
            uid,
            rid,
            start.parse().unwrap(),
            end.parse().unwrap(),
            note,
        );
        (manager.reserve(rsvp).await.unwrap(), manager)
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn query_reservations_should_work() {
        let (rsvp, manager) = make_reservation(migrated_pool.clone()).await;
        let query = ReservationQuery::new(
            "user_id1",
            "",
            "2022-12-01T12:00:00-0700".parse().unwrap(),
            "2022-12-31T12:00:00-0700".parse().unwrap(),
            ReservationStatus::Pending,
            1,
            10,
            false,
        );
        let rsvps = manager.query(query).await.unwrap();

        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvp, rsvps[0]);
    }
}
