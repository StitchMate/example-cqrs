use anyhow::anyhow;
use chrono::Utc;
use machines_rs::traits::TState;
use tracing::span;
use ulid::Ulid;

use crate::command::domain::account::{
    entity::{
        command::{AccountCommand, CreateAccountCommand},
        event::AccountEvent,
    },
    machine::context::AccountContext,
};

pub struct New;

impl TState<AccountContext> for New {
    fn entry(&mut self, _context: &mut AccountContext) {
        let root = span!(
            tracing::Level::INFO,
            "state entered",
            target = "AccountStateMachine",
            state = "New"
        );
        let _enter = root.enter();
    }

    fn exit(&mut self, context: &mut AccountContext) {
        let command: &AccountCommand = context.get_command().as_ref().unwrap();
        match command {
            AccountCommand::CreateAccount(CreateAccountCommand { email, password }) => {
                let root = span!(
                    tracing::Level::INFO,
                    "state exited",
                    target = "AccountStateMachine",
                    state = "New"
                );
                let _enter = root.enter();
                let span = span!(tracing::Level::INFO, "hashing password").entered();
                match context.get_services().hash_password(password.clone()) {
                    Ok(x) => {
                        span.exit();
                        context.set_event(AccountEvent::AccountCreated {
                            id: Ulid::new().to_string(),
                            email: email.clone(),
                            password_hash: x,
                            event_id: Ulid::new().to_string(),
                            created_at: Utc::now(),
                            event_version: "0.0.1".into(),
                        })
                    }
                    Err(_e) => context.set_error(anyhow!("Failed to hash password")),
                }
            }
        }
    }

    fn update(&mut self, _context: &mut AccountContext) {}
}
