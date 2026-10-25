//! Shared Postgres access layer.
//!
//! One submodule per table/view, each holding its row
//! model (`mod.rs`) and the associated [`Database`] methods (`impls.rs`).

pub mod changelog_counts_daily_aggregate;
pub mod measurements;
pub mod scores;
pub mod scores_daily_historic;
pub mod scores_per_minute;

use std::path::Path;

use chrono::{DateTime, Utc};
use color_eyre::{Result, eyre::Context};
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use sqlx::postgres::{PgPoolOptions, PgQueryResult, PgRow, PgStatement, PgTypeInfo};
use sqlx::{
    Describe, Either, Error as SqlxError, Execute, Executor, PgPool, Postgres, SqlStr, Transaction,
    pool::PoolConnection,
};

/// Postgres `TIMESTAMPTZ` mapped to a UTC timestamp.
pub type Timestamp = DateTime<Utc>;

#[derive(Debug)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(uri: &str) -> Result<Self> {
        tracing::debug!("Opening Postgres connection pool");
        let pool = PgPoolOptions::new().connect(uri).await?;
        tracing::info!("Postgres connection pool ready");

        Ok(Self { pool })
    }

    pub async fn acquire(&self) -> Result<PoolConnection<Postgres>> {
        tracing::trace!("Acquiring Postgres connection from pool");
        self.pool
            .acquire()
            .await
            .wrap_err("Failed to acquire a Postgres connection")
    }

    pub async fn begin(&self) -> Result<Transaction<'static, Postgres>> {
        tracing::trace!("Starting Postgres transaction");
        self.pool
            .begin()
            .await
            .wrap_err("Failed to start a Postgres transaction")
    }

    /// Disables chunkwise aggregation for the current query.
    ///
    /// Massively speeds up some aggregate-style functions that don't touch dates.
    pub async fn begin_with_chunkwise_aggregation_disabled(
        &self,
    ) -> Result<Transaction<'static, Postgres>> {
        tracing::trace!(
            "Starting Postgres transaction with timescaledb.enable_chunkwise_aggregation = off"
        );
        let mut trans = self.pool.begin().await.wrap_err(
            "Failed to start a Postgres transaction with disabled chunkwise aggregation",
        )?;

        sqlx::query("SET LOCAL timescaledb.enable_chunkwise_aggregation = off")
            .execute(&mut *trans)
            .await?;

        Ok(trans)
    }

    /// Runs the migrations found in `path`.
    ///
    /// The path is supplied by the caller because `sqlx::migrate!` resolves
    /// relative to the crate that invokes it.
    pub async fn migrate(&self, path: impl AsRef<Path>) -> Result<()> {
        tracing::trace!("Running migrations");
        let migrator = sqlx::migrate::Migrator::new(path.as_ref()).await?;
        migrator.run(&self.pool).await?;

        Ok(())
    }
}

impl<'d, 'p> Executor<'p> for &'d Database {
    type Database = Postgres;

    #[inline]
    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<'e, std::result::Result<Either<PgQueryResult, PgRow>, SqlxError>>
    where
        'p: 'e,
        E: Execute<'q, Self::Database>,
    {
        <&PgPool as Executor<'p>>::fetch_many(&self.pool, query)
    }

    #[inline]
    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, std::result::Result<Option<PgRow>, SqlxError>>
    where
        'p: 'e,
        E: Execute<'q, Self::Database>,
    {
        <&PgPool as Executor<'p>>::fetch_optional(&self.pool, query)
    }

    #[inline]
    fn prepare_with<'e>(
        self,
        sql: SqlStr,
        parameters: &'e [PgTypeInfo],
    ) -> BoxFuture<'e, std::result::Result<PgStatement, SqlxError>>
    where
        'p: 'e,
    {
        <&PgPool as Executor<'p>>::prepare_with(&self.pool, sql, parameters)
    }

    #[inline]
    fn describe<'e>(
        self,
        sql: SqlStr,
    ) -> BoxFuture<'e, std::result::Result<Describe<Self::Database>, SqlxError>>
    where
        'p: 'e,
    {
        <&PgPool as Executor<'p>>::describe(&self.pool, sql)
    }
}
