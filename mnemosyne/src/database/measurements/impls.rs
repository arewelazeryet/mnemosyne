use chrono::{TimeDelta, Utc};
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat, bail},
};
use sqlx::{query, query_as};

use crate::database::{Database, Timestamp};

use super::MeasurementEntry;

impl Database {
    pub async fn insert_measurement(&self, entry: MeasurementEntry) -> Result<()> {
        let result = query!(
            r#"
INSERT INTO measurements ( inserted_at, stable, lazer )
VALUES ( to_timestamp($1), $2, $3 )
            "#,
            entry.timestamp as f64,
            entry.stable,
            entry.lazer,
        )
        .execute(&*self)
        .await;

        match result {
            Ok(result) if result.rows_affected() > 0 => tracing::info!("Inserted measurement"),
            Ok(_) => tracing::debug!("Skipped duplicate measurement"),
            Err(error) => bail!("Database error: {error}"),
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_latest(&self) -> Result<MeasurementEntry> {
        tracing::debug!("Fetching peak lazer user count measurement");
        let result = query_as!(
            MeasurementEntry,
            r#"
SELECT
    EXTRACT(EPOCH FROM inserted_at)::BIGINT AS "timestamp!",
    stable,
    lazer
FROM measurements
ORDER BY inserted_at DESC
LIMIT 1
            "#,
        )
        .fetch_one(&*self)
        .await?;

        tracing::info!(
            stable = result.stable,
            lazer = result.lazer,
            "Found user peak at {}",
            result.timestamp
        );

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_count_peak(&self) -> Result<MeasurementEntry> {
        tracing::debug!("Fetching peak lazer user count measurement");
        let result = query_as!(
            MeasurementEntry,
            r#"
SELECT
    EXTRACT(EPOCH FROM inserted_at)::BIGINT AS "timestamp!",
    stable,
    lazer
FROM measurements
ORDER BY lazer DESC, inserted_at ASC
LIMIT 1
            "#,
        )
        .fetch_one(&*self)
        .await?;

        tracing::info!(
            stable = result.stable,
            lazer = result.lazer,
            "Found user peak at {}",
            result.timestamp
        );

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_ratio_peak(&self) -> Result<MeasurementEntry> {
        tracing::debug!("Fetching peak lazer ratio measurement");
        let result = query_as!(
            MeasurementEntry,
            r#"
SELECT
    EXTRACT(EPOCH FROM inserted_at)::BIGINT AS "timestamp!",
    stable,
    lazer
FROM measurements
WHERE (stable + lazer) > 5000
ORDER BY (lazer::DOUBLE PRECISION / NULLIF((stable + lazer)::DOUBLE PRECISION, 0)) DESC,
         inserted_at ASC
LIMIT 1
            "#,
        )
        .fetch_one(&*self)
        .await?;

        tracing::info!(
            stable = result.stable,
            lazer = result.lazer,
            "Found lazer% peak at {}",
            result.timestamp
        );

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_history_range(
        &self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Vec<MeasurementEntry>> {
        tracing::debug!("Fetching measurement history range");

        let result = query_as!(
            MeasurementEntry,
            r#"
SELECT
    EXTRACT(EPOCH FROM inserted_at)::BIGINT AS "timestamp!",
    stable,
    lazer
FROM measurements
WHERE inserted_at >= $1 AND inserted_at < $2
ORDER BY inserted_at ASC
            "#,
            start,
            end
        )
        .fetch_all(&*self)
        .await?;

        tracing::info!(len = result.len(), "Fetched history data");

        Ok(result)
    }

    pub async fn get_past_day(&self) -> Result<Vec<MeasurementEntry>> {
        let now = Utc::now();
        let start = now
            .checked_sub_signed(TimeDelta::hours(24))
            .wrap_err("Failed to compute past day start")?;

        self.get_history_range(start, now)
            .await
            .wrap_err("Failed to fetch past day")
    }
}
