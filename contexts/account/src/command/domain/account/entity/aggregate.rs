use crate::{
    command::domain::account::machine::{
        context::AccountContext, create_account_machine, states::States,
    },
    common::application::ports::outbound::account_services::AccountServices,
};

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cqrs_rs::domain::entity::{
    aggregate::Aggregate,
    event::{AggregateSnapshot, DomainEvent},
};
use struct_field_names_as_array::FieldNamesAsArray;
use tracing::span;
use ulid::Ulid;

use super::{command::AccountCommand, error::AccountError, event::AccountEvent};

#[derive(Clone, Debug, FieldNamesAsArray)]
pub struct AccountAggregate {
    pub id: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    pub password_hash: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_event: Option<AccountEvent>,
    pub applied_events: i32,
}

#[async_trait]
impl Aggregate for AccountAggregate {
    type Command = AccountCommand;
    type Event = AccountEvent;
    type Error = AccountError;
    type Services = Arc<dyn AccountServices + Sync + Send>;

    fn aggregate_type() -> String {
        "Account".to_string()
    }

    fn aggregate_id(&self) -> Option<String> {
        return self.id.clone();
    }

    async fn handle(
        &self,
        command: Self::Command,
        services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        let root = span!(tracing::Level::INFO, "handle", target = "AccountAggregate");
        let _enter = root.enter();
        span!(tracing::Level::INFO, "aggregate has received command");
        let mut context: AccountContext = match &self.last_event {
            Some(_) => AccountContext::new(services.clone(), Some(self.clone())),
            None => AccountContext::new(services.clone(), None),
        };
        span!(tracing::Level::INFO, "state machine context constructed");
        let mut machine = match &self.last_event {
            Some(x) => match x {
                AccountEvent::AccountCreated { .. } => create_account_machine(States::Created),
            },
            None => create_account_machine(States::New),
        };
        span!(tracing::Level::INFO, "state machine reconstituted");
        context.set_command(command.clone());
        let machine_span = span!(tracing::Level::INFO, "machine executed").entered();
        machine.decide(&mut context);
        machine_span.exit();
        return match context.get_event() {
            Some(x) => Ok(vec![x.clone()]),
            None => Err(AccountError::StateMachineTransitionFail(command)),
        };
    }

    fn apply(&mut self, event: Self::Event) {
        self.applied_events += 1;
        match &event {
            AccountEvent::AccountCreated {
                id,
                email,
                password_hash,
                created_at,
                ..
            } => {
                self.id = Some(id.clone());
                self.email = Some(email.clone());
                self.password_hash = Some(password_hash.clone());
                self.created_at = Some(created_at.clone());
                self.last_event = Some(event);
            }
        }
    }

    fn apply_snapshot(&mut self, snapshot: AggregateSnapshot<Self>) {
        let payload = snapshot.payload;
        self.id = payload.id;
        self.email = payload.email;
        self.password_hash = payload.password_hash;
        self.created_at = payload.created_at;
        self.last_event = payload.last_event;
        self.status = payload.status;
    }

    fn snapshot(&mut self) -> Option<AggregateSnapshot<Self>> {
        if self.applied_events >= 10 {
            let snapshot: AggregateSnapshot<Self> = AggregateSnapshot {
                aggregate_id: self.aggregate_id().unwrap(),
                aggregate_type: Self::aggregate_type(),
                payload: self.clone(),
                last_sequence: self.last_event.as_ref().unwrap().event_id(),
                snapshot_id: Ulid::new().to_string(),
                timestamp: Utc::now(),
            };
            return Some(snapshot);
        }
        return None;
    }
}

impl Default for AccountAggregate {
    fn default() -> Self {
        AccountAggregate {
            id: None,
            email: None,
            status: None,
            password_hash: None,
            created_at: None,
            last_event: None,
            applied_events: 0,
        }
    }
}
