use crate::command::domain::account::entity::aggregate::AccountAggregate;

use async_graphql::{SimpleObject, InputObject};
use chrono::{DateTime, Utc};
use validator::Validate;

#[derive(Clone, SimpleObject)]
#[graphql(name = "Account")]
pub struct GraphQLAccount {
    pub id: Option<String>,
    pub email: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<AccountAggregate> for GraphQLAccount {
    fn from(value: AccountAggregate) -> Self {
        return GraphQLAccount {
            id: value.id,
            email: value.email,
            created_at: value.created_at,
        };
    }
}

#[derive(Clone, InputObject, Validate)]
#[graphql(name = "CreateAccountInput")]
pub struct GraphQLCreateAccountInput {
    #[validate(email)]
    pub email: String,
    pub password: String
}

impl GraphQLCreateAccountInput {
    pub fn new(email: String, password: String) -> Self {
        return Self { email, password };
    }
}

