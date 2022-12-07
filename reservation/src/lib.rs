mod manager;
use async_trait::async_trait;
pub type ReservationId = String;
pub type UserId = String;
pub type ResourceId = String;

#[async_trait]
pub trait Rsvp {
    /// generate a reservation
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error>;

    /// if current status is pending, change it to confirmed
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error>;

    /// update note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error>;

    /// delete reservation
    async fn delete(&self, id: ReservationId) -> Result<(), abi::Error>;

    /// get reservation by id
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error>;

    /// query reservations
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error>;
}
