use crate::{
    command::{
        application::account::ports::{
            inbound::create_account::CreateAccountUseCase,
            outbound::repository::AccountEventRepository,
        },
        domain::account::entity::{
            aggregate::AccountAggregate, command::CreateAccountCommand, error::AccountError,
        },
    },
    common::application::ports::outbound::account_services::AccountServices,
};

use std::collections::HashMap;
use std::{sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use cqrs_rs::domain::entity::{aggregate::Aggregate, event::DomainEvent, event::EventEnvelope};
use tracing::span;

pub trait ServiceTrait<O: From<AccountAggregate>>: CreateAccountUseCase<O> {}

pub struct AccountService<T, Q> {
    services: Arc<dyn AccountServices + Sync + Send>,
    repository: Arc<dyn AccountEventRepository<T, Q> + Sync + Send>,
}

impl<T, Q> AccountService<T, Q> {
    pub fn new(
        services: Arc<dyn AccountServices + Sync + Send>,
        repository: Arc<dyn AccountEventRepository<T, Q> + Sync + Send>,
    ) -> Self {
        return Self {
            services,
            repository,
        };
    }
}

#[async_trait]
impl<O, T, Q> CreateAccountUseCase<O> for AccountService<T, Q>
where
    O: From<AccountAggregate>,
{
    async fn create_account(
        &self,
        command: CreateAccountCommand,
        _fields: Vec<&str>,
    ) -> Result<O, anyhow::Error> {
        let root = span!(
            tracing::Level::INFO,
            "create_account",
            target = "AccountService"
        );
        let _enter = root.enter();
        let mut aggregate = AccountAggregate::default();
        let email = command.email.clone();
        let exists = self.repository.email_exists(email.clone()).await?;
        if exists {
            return Err(AccountError::AccountExists(email).into());
        }
        let events = aggregate.handle(command.into(), &self.services).await?;
        events
            .iter()
            .for_each(|event| aggregate.apply(event.clone()));
        if aggregate.aggregate_id().is_none() {
            return Err(AccountError::UnknownError.into());
        }
        let wrapped_events = events
            .iter()
            .map(|x| EventEnvelope::<AccountAggregate> {
                aggregate_id: aggregate.aggregate_id().unwrap(),
                aggregate_type: "account".into(),
                sequence: x.event_id(),
                payload: x.clone(),
                metadata: HashMap::new(),
                timestamp: Utc::now(),
            })
            .collect();
        self.repository.store_events(wrapped_events).await?;
        match aggregate.snapshot() {
            Some(x) => self.repository.store_snapshot(x).await?,
            _ => {}
        }
        return Ok(aggregate.into());
    }
}

impl<O: From<AccountAggregate>, T, Q> ServiceTrait<O> for AccountService<T, Q> {}
