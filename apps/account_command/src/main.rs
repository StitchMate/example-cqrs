use std::str::FromStr;
use std::sync::Arc;

use account::command::application::account::ports::outbound::repository::AccountEventRepository;
use account::{
    command::{
        application::account::service::account::AccountService,
        infrastructure::{
            adapters::{
                inbound::graphql::GraphQLAccountCommandAdapter,
                outbound::sqlite::SQLiteAccountRepository,
            },
            dtos::transport::nats::NATSAccountEvent,
        },
    },
    common::{
        application::ports::outbound::account_services,
        infrastructure::adapters::outbound::account_services::argon2::AccountServices,
    },
};
use anyhow::anyhow;
use cqrs_rs::infrastructure::{
    adapter::secondary::storage::sqlite::SqliteConnector, dto::transport::nats::NATSEventEnvelope,
};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteJournalMode;
use sqlx::sqlite::SqliteSynchronous;
use sqlx::{Pool, Sqlite};
use tokio::signal;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    opentelemetry::global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_endpoint("localhost:6831")
        .with_service_name("account-command")
        .install_simple()?;
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry().with(telemetry).try_init()?;
    let conn: Result<Pool<Sqlite>, anyhow::Error> = sqlx::Pool::connect_with(
        SqliteConnectOptions::from_str("sqlite:data.db")?
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true)
            .foreign_keys(false)
            .synchronous(SqliteSynchronous::Normal),
    )
    .await
    .map_err(|e| anyhow!(e));
    let connector = SqliteConnector::new(conn).await.unwrap();
    let repository: Arc<
        dyn AccountEventRepository<NATSEventEnvelope<NATSAccountEvent>, String>
            + Send
            + Sync
            + 'static,
    > = Arc::new(SQLiteAccountRepository {
        connector: connector,
    });
    match repository
        .migrate("../../contexts/account/src/command/migrations".into())
        .await
    {
        Err(e) => {
            println!("ERROR: {:?}", e);
            std::process::exit(1)
        }
        _ => {}
    }
    let services: Arc<dyn account_services::AccountServices + Sync + Send> =
        Arc::new(AccountServices::new());
    let service: Arc<AccountService<NATSEventEnvelope<NATSAccountEvent>, String>> =
        Arc::new(AccountService::new(services.clone(), repository.clone()));

    GraphQLAccountCommandAdapter::new(service).run().await?;

    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("🎩 Ctrl-C received, shutting down");
        }
        _ = sigterm.recv() => {
            println!("terminate signal received, shutting down");
        }
    }

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
