use chrono::{DateTime, Utc};
use cqrs_rs::domain::entity::event::DomainEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum AccountEvent {
    AccountCreated {
        id: String,
        email: String,
        password_hash: String,
        created_at: DateTime<Utc>,
        event_version: String,
        event_id: String,
    }
}

impl DomainEvent for AccountEvent {
    fn event_type(&self) -> String {
        match self {
            AccountEvent::AccountCreated { .. } => "AccountCreated".into()
        }
    }

    fn event_version(&self) -> String {
        match self {
            AccountEvent::AccountCreated { event_version, .. } => event_version.into()
        }
    }

    fn event_id(&self) -> String {
        match self {
            AccountEvent::AccountCreated { event_id, .. } => event_id.into()
        }
    }
}