use crate::command::domain::account::entity::{aggregate::AccountAggregate, command::CreateAccountCommand};

use async_trait::async_trait;

#[async_trait]
pub trait CreateAccountUseCase<O>
where
    O: From<AccountAggregate>,
{
    async fn create_account(
        &self,
        account: CreateAccountCommand,
        fields: Vec<&str>
    ) -> Result<O, anyhow::Error>;
}