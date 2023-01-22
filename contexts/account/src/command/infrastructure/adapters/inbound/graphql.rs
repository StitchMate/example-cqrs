use crate::command::{
    application::account::service::account::ServiceTrait,
    domain::account::entity::command::CreateAccountCommand,
    infrastructure::dtos::transport::graphql::{GraphQLAccount, GraphQLCreateAccountInput},
};

use std::sync::Arc;

use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer, guard};
use async_graphql::{http::GraphiQLSource, Context, EmptySubscription, Object, Result, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use tracing::span;
use validator::Validate;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[graphql(visible = false)]
    async fn _version(&self) -> String {
        return VERSION.into();
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_account(
        &self,
        ctx: &Context<'_>,
        input: GraphQLCreateAccountInput,
    ) -> Result<GraphQLAccount> {
        let service = ctx
            .data::<Arc<dyn ServiceTrait<GraphQLAccount> + Sync + Send>>()
            .unwrap();
        if input.validate().is_err() {
            return Err(input.validate().unwrap_err().into());
        }
        let command = CreateAccountCommand {
            email: input.email,
            password: input.password,
        };
        let result = service.create_account(command.into(), vec![]).await;
        match result {
            Ok(x) => return Ok(x),
            Err(e) => return Err(e.into()),
        }
    }
}

#[derive(Clone)]
pub struct GraphQLAccountCommandAdapter {
    schema: Schema<QueryRoot, MutationRoot, EmptySubscription>
}

async fn index(
    schema: web::Data<Schema<QueryRoot, MutationRoot, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let root = span!(tracing::Level::INFO, "graphql_request_received");
    let _enter = root.enter();
    schema.execute(req.into_inner()).await.into()
}

async fn gql_playgound() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/").finish())
}

impl GraphQLAccountCommandAdapter {
    pub fn new(service: Arc<dyn ServiceTrait<GraphQLAccount> + Send + Sync>) -> Self {
        let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).enable_federation()
            .data(service)
            .finish();
        return Self {
            schema
        }
    }
    pub async fn run(self) -> Result<(), anyhow::Error> {
        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(self.schema.clone()))
                .service(web::resource("/").guard(guard::Post()).to(index))
                .service(web::resource("/").guard(guard::Get()).to(gql_playgound))
        })
        .bind("0.0.0.0:3000")?
        .run()
        .await
        .map_err(|e| e.into())
    }
}
