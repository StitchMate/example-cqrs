use crate::command::domain::account::entity::aggregate::AccountAggregate;

use async_trait::async_trait;
use cqrs_rs::domain::entity::event::EventEnvelope;

#[async_trait]
pub trait SendEvent<I>
where
    I: Into<EventEnvelope<AccountAggregate>>,
{
    async fn send_event(&self, event: I) -> Result<(), anyhow::Error>;
}