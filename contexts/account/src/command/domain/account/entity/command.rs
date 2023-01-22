use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum AccountCommand {
    CreateAccount(CreateAccountCommand)
}

impl AccountCommand {
    pub fn to_string(&self) -> String {
        match self {
            Self::CreateAccount { .. } => "CreateAccount".into()
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateAccountCommand {
    pub email: String,
    pub password: String
}

impl Into<AccountCommand> for CreateAccountCommand {
    fn into(self) -> AccountCommand {
        AccountCommand::CreateAccount(self)
    }
}