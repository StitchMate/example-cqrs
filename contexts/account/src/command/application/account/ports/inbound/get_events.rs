use crate::command::domain::account::entity::aggregate::AccountAggregate;

use async_trait::async_trait;
use cqrs_rs::domain::entity::event::EventEnvelope;

#[async_trait]
pub trait GetEvents<O>
where
    O: Into<EventEnvelope<AccountAggregate>>,
{
    async fn get_events(&self) -> Result<Vec<O>, anyhow::Error>;
}