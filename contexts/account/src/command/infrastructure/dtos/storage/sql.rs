use crate::command::domain::account::entity::{aggregate::AccountAggregate, event::AccountEvent};

use chrono::{serde::ts_seconds, serde::ts_seconds_option, DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event_type")]
pub enum SQLAccountEvent {
    AccountCreated {
        id: String,
        event_id: String,
        event_version: String,
        email: String,
        password_hash: String,
        #[serde(with = "ts_seconds")]
        created_at: DateTime<Utc>
    }
}

impl Default for SQLAccountEvent {
    fn default() -> Self {
        return Self::AccountCreated {
            id: "".into(),
            event_version: "0.0.0".to_string(),
            event_id: Ulid::new().to_string(),
            email: "".into(),
            password_hash: "".into(),
            created_at: Utc::now()
        };
    }
}

impl Into<Option<AccountEvent>> for SQLAccountEvent {
    fn into(self) -> Option<AccountEvent> {
        match self {
            Self::AccountCreated {
                id,
                event_id,
                event_version,
                email,
                password_hash,
                created_at
            } => Some(AccountEvent::AccountCreated {
                id,
                event_id,
                event_version,
                email,
                password_hash,
                created_at
            })
        }
    }
}

impl From<AccountEvent> for SQLAccountEvent {
    fn from(u: AccountEvent) -> Self {
        match u {
            AccountEvent::AccountCreated {
                id,
                event_id,
                event_version,
                email,
                password_hash,
                created_at
            } => Self::AccountCreated {
                id,
                event_id,
                event_version,
                email,
                password_hash,
                created_at,
            }
        }
    }
}

impl Into<AccountEvent> for SQLAccountEvent {
    fn into(self) -> AccountEvent {
        match self {
            Self::AccountCreated {
                id,
                event_id,
                event_version,
                email,
                password_hash,
                created_at
            } => AccountEvent::AccountCreated {
                id,
                event_id,
                event_version,
                email,
                password_hash,
                created_at
            },
        }
    }
}

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct SQLAccountAggregate {
    id: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    #[serde(with = "ts_seconds_option")]
    pub created_at: Option<DateTime<Utc>>,
    last_event: Option<SQLAccountEvent>,
}

impl Into<AccountAggregate> for SQLAccountAggregate {
    fn into(self) -> AccountAggregate {
        return AccountAggregate {
            id: self.id,
            email: self.email,
            password_hash: self.password_hash,
            created_at: self.created_at,
            last_event: self.last_event.map(|x| x.into()),
            ..Default::default()
        };
    }
}

impl From<AccountAggregate> for SQLAccountAggregate {
    fn from(value: AccountAggregate) -> Self {
        return SQLAccountAggregate {
            id: value.id,
            email: value.email,
            password_hash: value.password_hash,
            created_at: value.created_at,
            last_event: value.last_event.map(|x| x.into()),
        };
    }
}