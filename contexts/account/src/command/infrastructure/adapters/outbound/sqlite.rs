use crate::command::{
    application::account::ports::outbound::repository::{
        AccountEventRepository, AccountRepository,
    },
    domain::account::entity::{aggregate::AccountAggregate, error::AccountError},
    infrastructure::dtos::storage::sql::{SQLAccountAggregate, SQLAccountEvent},
};

use std::sync::Arc;

use async_trait::async_trait;
use cqrs_rs::{
    application::port::outbound::{event_bus::EventBus, event_repository::EventRepository},
    domain::entity::event::{AggregateSnapshot, DomainEvent, EventEnvelope},
    infrastructure::{
        adapter::secondary::storage::sqlite::SqliteConnector,
        dto::storage::sql::{SQLAggregateSnapshot, SQLEventEnvelope},
    },
};
use serde_json::json;
use sqlx::{migrate::Migrator, Row, Sqlite};
use tracing::{span, Instrument};

const EVENT_TABLE_NAME: &str = "account_events";
const SNAPSHOT_TABLE_NAME: &str = "account_snapshots";
const OUTBOX_TABLE_NAME: &str = "account_outbox_events";

#[derive(Clone)]
pub struct SQLiteAccountRepository {
    pub connector: Arc<SqliteConnector>,
}

impl<
        T: From<EventEnvelope<AccountAggregate>> + Into<EventEnvelope<AccountAggregate>> + Into<Q>,
        Q,
    > AccountEventRepository<T, Q> for SQLiteAccountRepository
{
}

#[async_trait]
impl AccountRepository for SQLiteAccountRepository {
    async fn email_exists(&self, email: String) -> Result<bool, anyhow::Error> {
        let root = span!(
            tracing::Level::INFO,
            "email_exists",
            target = "AccountEventRepository",
            implementation = "SQLiteAccountRepository"
        );
        let _enter = root.enter();
        let query = format!(
            "SELECT COUNT(*) FROM {} WHERE json_extract(payload, '$.email') = ?1",
            EVENT_TABLE_NAME
        );
        let plan = sqlx::query::<Sqlite>(&query).bind(email);
        let execute_span = span!(tracing::Level::INFO, "query execute");
        let results = plan
            .fetch_one(&self.connector.pool)
            .instrument(execute_span)
            .await;
        match results {
            Err(e) => return Err(e.into()),
            Ok(x) => {
                let count: i32 = x.get(0);
                if count > 0 {
                    return Ok(true);
                }
                return Ok(false);
            }
        };
    }
    async fn retrieve_aggregate_id_for_email(
        &self,
        email: String,
    ) -> Result<String, anyhow::Error> {
        let query = format!(
            "SELECT aggregate_id FROM {} WHERE json_extract(payload, '$.email') = ?1 ORDER BY sequence DESC LIMIT 1",
            EVENT_TABLE_NAME
        );

        let plan = sqlx::query::<Sqlite>(&query).bind(email);
        let results = plan.fetch_one(&self.connector.pool).await;
        match results {
            Err(e) => return Err(e.into()),
            Ok(x) => {
                let id: String = x.get(0);
                return Ok(id);
            }
        };
    }
}

#[async_trait]
impl<
        T: From<EventEnvelope<AccountAggregate>> + Into<EventEnvelope<AccountAggregate>> + Into<Q>,
        Q,
    >
    EventRepository<
        EventEnvelope<AccountAggregate>,
        T,
        Q,
        EventEnvelope<AccountAggregate>,
        AggregateSnapshot<AccountAggregate>,
        AggregateSnapshot<AccountAggregate>,
    > for SQLiteAccountRepository
{
    async fn migrate(&self, path: String) -> Result<(), anyhow::Error> {
        let m = Migrator::new(std::path::Path::new(&path)).await?;
        m.run(&self.connector.pool).await.map_err(|e| e.into())
    }
    async fn store_events(
        &self,
        events: Vec<EventEnvelope<AccountAggregate>>,
    ) -> Result<(), anyhow::Error> {
        let root = span!(
            tracing::Level::INFO,
            "store_events",
            target = "AccountEventRepository",
            implementation = "SQLiteAccountRepository"
        );
        let _enter = root.enter();
        let fields = vec![
            "aggregate_type",
            "aggregate_id",
            "sequence",
            "event_type",
            "event_version",
            "payload",
            "metadata",
            "timestamp",
        ];
        let placeholders: Vec<String> = (0..fields.len())
            .map(|x| format!("?{}", (x + 1).to_string()))
            .collect();
        let placeholder_str = placeholders.join(", ");
        let query = format!(
            "INSERT INTO {} ({}) VALUES ( {} )",
            EVENT_TABLE_NAME,
            fields.join(", "),
            placeholder_str
        );
        let outbox_query = format!(
            "INSERT INTO {} ({}) VALUES ( {} )",
            OUTBOX_TABLE_NAME,
            fields.join(", "),
            placeholder_str
        );
        let mut results = vec![];
        for x in events {
            let mut tx = self.connector.pool.begin().await?;
            let plan = sqlx::query::<Sqlite>(&query);
            let outbox_plan = sqlx::query::<Sqlite>(&outbox_query);
            let insert_span = span!(
                tracing::Level::INFO,
                "insert event",
                event = format!("{:?}", x)
            );
            let enum_sql: SQLAccountEvent = x.payload.clone().into();
            let insert = plan
                .bind(&x.aggregate_type)
                .bind(&x.aggregate_id)
                .bind(&x.sequence)
                .bind(&x.payload.event_type())
                .bind(&x.payload.event_version())
                .bind(json!(enum_sql).to_string())
                .bind(json!(x.metadata).to_string())
                .bind(&x.timestamp.to_rfc3339())
                .execute(&mut tx)
                .instrument(insert_span)
                .await;
            let insert_outbox_span = span!(
                tracing::Level::INFO,
                "insert outbox event",
                event = format!("{:?}", x)
            );
            let outbox_insert = outbox_plan
                .bind(&x.aggregate_type)
                .bind(&x.aggregate_id)
                .bind(&x.sequence)
                .bind(&x.payload.event_type())
                .bind(&x.payload.event_version())
                .bind(json!(enum_sql).to_string())
                .bind(json!(x.metadata).to_string())
                .bind(&x.timestamp.to_rfc3339())
                .execute(&mut tx)
                .instrument(insert_outbox_span)
                .await;
            if outbox_insert.is_err() {
                results.push(outbox_insert)
            }
            tx.commit().await?;
            results.push(insert);
        }
        let mut err: Vec<anyhow::Error> = vec![];
        for result in results {
            match result {
                Err(e) => err.push(e.into()),
                _ => {}
            }
        }
        if err.len() > 0 {
            return Err(AccountError::UnknownError.into());
        }
        return Ok(());
    }

    async fn retrieve_events(
        &self,
        aggregate_id: String,
        after: Option<String>,
    ) -> Result<Vec<EventEnvelope<AccountAggregate>>, anyhow::Error> {
        let fields = vec![
            "aggregate_type",
            "aggregate_id",
            "sequence",
            "event_type",
            "event_version",
            "payload",
            "metadata",
            "timestamp",
        ];
        let query = match after {
            None => format!(
                "SELECT {} FROM {} WHERE aggregate_id = ?1",
                fields.join(", "),
                EVENT_TABLE_NAME
            ),
            Some(_) => format!(
                "SELECT {} FROM {} WHERE aggregate_id = ?1 AND sequence > ?2 ORDER BY sequence ASC",
                fields.join(", "),
                EVENT_TABLE_NAME
            ),
        };
        let mut plan = sqlx::query_as::<Sqlite, SQLEventEnvelope<SQLAccountEvent>>(&query);
        plan = match after {
            None => plan.bind(aggregate_id),
            Some(x) => plan.bind(aggregate_id).bind(x),
        };
        let results = plan.fetch_all(&self.connector.pool).await;
        match results {
            Err(e) => return Err(e.into()),
            _ => {}
        };
        let mut resp: Vec<EventEnvelope<AccountAggregate>> = vec![];
        for env in results.unwrap() {
            let x = env.into();
            resp.push(x)
        }
        return Ok(resp);
    }

    async fn store_snapshot(
        &self,
        snapshot: AggregateSnapshot<AccountAggregate>,
    ) -> Result<(), anyhow::Error> {
        println!("Storing snapshot");
        let fields = vec![
            "aggregate_type",
            "aggregate_id",
            "payload",
            "last_sequence",
            "snapshot_id",
            "timestamp",
        ];
        let placeholders: Vec<String> = (0..fields.len())
            .map(|x| format!("?{}", (x + 1).to_string()))
            .collect();
        let placeholder_str = placeholders.join(", ");
        let query = format!(
            "INSERT INTO {} ({}) VALUES ( {} )",
            SNAPSHOT_TABLE_NAME,
            fields.join(", "),
            placeholder_str
        );
        let plan = sqlx::query::<Sqlite>(&query);
        let enum_sql: SQLAccountAggregate = snapshot.payload.clone().into();
        let insert = plan
            .bind(snapshot.aggregate_type)
            .bind(snapshot.aggregate_id)
            .bind(json!(enum_sql).to_string())
            .bind(snapshot.last_sequence)
            .bind(snapshot.snapshot_id)
            .bind(snapshot.timestamp)
            .fetch_optional(&self.connector.pool)
            .await;
        match insert {
            Err(_e) => {
                println!("INSERT ERROR {:?}", _e);
                return Err(AccountError::UnknownError.into());
            }
            _ => return Ok(()),
        }
    }

    async fn retrieve_latest_snapshot(
        &self,
        aggregate_id: String,
    ) -> Result<Option<AggregateSnapshot<AccountAggregate>>, anyhow::Error> {
        let fields = vec![
            "aggregate_type",
            "aggregate_id",
            "payload",
            "last_sequence",
            "snapshot_id",
            "timestamp",
        ];
        let query = format!(
            "SELECT {} FROM {} WHERE aggregate_id = ?1 ORDER BY snapshot_id DESC LIMIT 1",
            fields.join(", "),
            SNAPSHOT_TABLE_NAME
        );
        let plan = sqlx::query_as::<Sqlite, SQLAggregateSnapshot<SQLAccountAggregate>>(&query)
            .bind(aggregate_id);
        let result = plan.fetch_optional(&self.connector.pool).await;
        match result {
            Err(e) => return Err(e.into()),
            _ => {}
        };
        match result.unwrap() {
            None => Ok(None),
            Some(x) => Ok(Some(x.into())),
        }
    }

    async fn send_and_delete_outbox_event(
        &self,
        event: EventEnvelope<AccountAggregate>,
        bus: &Arc<
            dyn EventBus<EventEnvelope<AccountAggregate>, T, Q, EventEnvelope<AccountAggregate>>
                + Sync
                + Send,
        >,
    ) -> Result<(), anyhow::Error> {
        let query = format!("DELETE FROM {} WHERE sequence = ?1", OUTBOX_TABLE_NAME);
        let mut plan = sqlx::query::<Sqlite>(&query);
        plan = plan.bind(event.sequence.clone());
        let mut tx = self.connector.pool.begin().await?;
        bus.send_event(event).await?;
        let result = plan.execute(&mut tx).await;
        match result {
            Err(e) => {
                tx.rollback().await?;
                return Err(e.into());
            }
            _ => {
                tx.commit().await?;
                return Ok(());
            }
        }
    }

    async fn retrieve_outbox_events(
        &self,
    ) -> Result<Vec<EventEnvelope<AccountAggregate>>, anyhow::Error> {
        let fields = vec![
            "aggregate_type",
            "aggregate_id",
            "sequence",
            "event_type",
            "event_version",
            "payload",
            "metadata",
            "timestamp",
        ];
        let query = format!("SELECT {} FROM {}", fields.join(", "), OUTBOX_TABLE_NAME);
        let plan = sqlx::query_as::<Sqlite, SQLEventEnvelope<SQLAccountEvent>>(&query);
        let results = plan.fetch_all(&self.connector.pool).await;
        match results {
            Err(e) => return Err(e.into()),
            _ => {}
        };
        let mut resp: Vec<EventEnvelope<AccountAggregate>> = vec![];
        for env in results.unwrap() {
            let x = env.into();
            resp.push(x)
        }
        return Ok(resp);
    }
}
