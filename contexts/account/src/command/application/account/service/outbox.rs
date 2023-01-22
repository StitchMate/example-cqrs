use crate::command::{domain::account::entity::aggregate::AccountAggregate, application::account::ports::{inbound::{get_events::GetEvents, send_event::SendEvent}}};

use std::sync::Arc;

use async_trait::async_trait;
use cqrs_rs::{domain::entity::event::{EventEnvelope, AggregateSnapshot}, application::port::outbound::{event_bus::EventBus, event_repository::EventRepository}};


pub struct AccountOutboxService<T, Q> {
    repository: Arc<
        dyn EventRepository<
                EventEnvelope<AccountAggregate>,
                T,
                Q,
                EventEnvelope<AccountAggregate>,
                AggregateSnapshot<AccountAggregate>,
                AggregateSnapshot<AccountAggregate>,
            > + Sync
            + Send,
    >,
    bus: Arc<
        dyn EventBus<EventEnvelope<AccountAggregate>, T, Q, EventEnvelope<AccountAggregate>>
            + Sync
            + Send,
    >,
}

impl<T, Q> AccountOutboxService<T, Q> {
    pub fn new(
        repository: Arc<
            dyn EventRepository<
                    EventEnvelope<AccountAggregate>,
                    T,
                    Q,
                    EventEnvelope<AccountAggregate>,
                    AggregateSnapshot<AccountAggregate>,
                    AggregateSnapshot<AccountAggregate>,
                > + Sync
                + Send,
        >,
        bus: Arc<
            dyn EventBus<EventEnvelope<AccountAggregate>, T, Q, EventEnvelope<AccountAggregate>>
                + Sync
                + Send,
        >,
    ) -> Self {
        return Self { repository, bus };
    }
}

#[async_trait]
impl<T, Q> GetEvents<EventEnvelope<AccountAggregate>> for AccountOutboxService<T, Q> {
    async fn get_events(&self) -> Result<Vec<EventEnvelope<AccountAggregate>>, anyhow::Error> {
        self.repository.retrieve_outbox_events().await
    }
}

#[async_trait]
impl<T, Q> SendEvent<EventEnvelope<AccountAggregate>> for AccountOutboxService<T, Q> {
    async fn send_event(
        &self,
        event: EventEnvelope<AccountAggregate>,
    ) -> Result<(), anyhow::Error> {
        self.repository
            .send_and_delete_outbox_event(event, &self.bus)
            .await
    }
}