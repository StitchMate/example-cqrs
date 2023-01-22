use thiserror::Error;
use super::command::AccountCommand;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("account with email `{0}` does not exist")]
    AccountNotExists(String),
    #[error("account with email `{0}` already exists")]
    AccountExists(String),
    #[error("state machine failed to emit event for command `{0:?}`")]
    StateMachineTransitionFail(AccountCommand),
    #[error("unknown error occured")]
    UnknownError
}