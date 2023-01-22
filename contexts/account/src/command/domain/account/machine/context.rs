use crate::{command::domain::account::entity::{command::AccountCommand, event::AccountEvent, aggregate::AccountAggregate}, common::application::ports::outbound::account_services::AccountServices};

use std::sync::Arc;

#[derive(Debug)]
pub struct AccountContext {
    command: Option<AccountCommand>,
    event: Option<AccountEvent>,
    error: Option<anyhow::Error>,
    current_state: Option<AccountAggregate>,
    services: Arc<dyn Send + Sync + AccountServices>,
}

impl AccountContext {
    pub fn new(
        services: Arc<dyn AccountServices + Send + Sync>,
        current_state: Option<AccountAggregate>,
    ) -> Self {
        return Self {
            current_state,
            command: None,
            event: None,
            error: None,
            services
        };
    }
    pub fn get_event(&self) -> &Option<AccountEvent> {
        return &self.event;
    }
    pub fn set_event(&mut self, event: AccountEvent) {
        self.event = Some(event);
    }
    pub fn get_command(&self) -> &Option<AccountCommand> {
        return &self.command;
    }
    pub fn set_command(&mut self, command: AccountCommand) {
        self.command = Some(command);
    }
    pub fn get_current_state(&self) -> &Option<AccountAggregate> {
        return &self.current_state;
    }
    pub fn get_error(&self) -> &Option<anyhow::Error> {
        return &self.error;
    }
    pub fn set_error(&mut self, error: anyhow::Error) {
        self.error = Some(error);
    }
    pub fn get_services(&self) -> Arc<dyn Send + Sync + AccountServices> {
        return self.services.clone();
    }
}