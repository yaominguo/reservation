mod error;
mod manager;
use async_trait::async_trait;
pub use error::ReservationError;
pub type ReservationId = String;
pub type UserId = String;
pub type ResourceId = String;

#[async_trait]
pub trait Rsvp {
    /// generate a reservation
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, ReservationError>;

    /// if current status is pending, change it to confirmed
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;

    /// update note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError>;

    /// delete reservation
    async fn delete(&self, id: ReservationId) -> Result<(), ReservationError>;

    /// get reservation by id
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;

    /// query reservations
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError>;
}
