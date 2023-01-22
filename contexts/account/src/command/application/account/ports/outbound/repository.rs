use crate::command::domain::account::entity::aggregate::AccountAggregate;

use async_trait::async_trait;
use cqrs_rs::{
    application::port::outbound::event_repository::EventRepository,
    domain::entity::event::{AggregateSnapshot, EventEnvelope},
};

#[async_trait]
pub trait AccountRepository {
    async fn email_exists(&self, email: String) -> Result<bool, anyhow::Error>;
    async fn retrieve_aggregate_id_for_email(&self, email: String)
        -> Result<String, anyhow::Error>;
}

pub trait AccountEventRepository<T, Q>:
    EventRepository<
        EventEnvelope<AccountAggregate>,
        T,
        Q,
        EventEnvelope<AccountAggregate>,
        AggregateSnapshot<AccountAggregate>,
        AggregateSnapshot<AccountAggregate>,
    > + AccountRepository
{
}
