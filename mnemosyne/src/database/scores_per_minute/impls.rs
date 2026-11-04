use color_eyre::Result;
use sqlx::query_as;

use crate::database::Database;

use super::ScorePerMinute;

impl Database {
    pub async fn get_minute_historic_graphs(&self) -> Result<Vec<ScorePerMinute>> {
        tracing::debug!("Fetching minute historic graphs");

        let data = query_as!(
            ScorePerMinute,
            r#"
SELECT
    COALESCE(EXTRACT(EPOCH FROM bucket)::BIGINT, 0) AS "bucket!",
    ruleset_id AS "ruleset_id!",
    client_type AS "client_type!",

    hll_cardinality(user_hll) AS "unique_user_count!",
    hll_cardinality(beatmap_hll) AS "unique_beatmap_count!",

    total_scores_per_min AS "total_scores_per_min!",
    replays_per_min AS "replays_per_min!",
    perfect_combos_per_min AS "perfect_combos_per_min!",

    min_pp AS "min_pp!",
    max_pp AS "max_pp!",
    sum_pp AS "sum_pp!",

    sum_total_score AS "sum_total_score!",
    sum_classic_total_score AS "sum_classic_total_score!",
    sum_legacy_total_score AS "sum_legacy_total_score!",

    max_classic_total_score AS "max_classic_total_score!",
    max_legacy_total_score AS "max_legacy_total_score!",

    sum_accuracy AS "sum_accuracy!",
    peak_combo AS "peak_combo!"
FROM scores_per_minute;
            "#
        )
        .fetch_all(&*self)
        .await?;

        Ok(data)
    }

    /// Fetches the last 24 hours of minute buckets.
    pub async fn get_recent_minute_historic_graphs(&self) -> Result<Vec<ScorePerMinute>> {
        tracing::debug!("Fetching recent minute historic graphs");

        let data = query_as!(
            ScorePerMinute,
            r#"
SELECT
    COALESCE(EXTRACT(EPOCH FROM bucket)::BIGINT, 0) AS "bucket!",
    ruleset_id AS "ruleset_id!",
    client_type AS "client_type!",

    hll_cardinality(user_hll) AS "unique_user_count!",
    hll_cardinality(beatmap_hll) AS "unique_beatmap_count!",

    total_scores_per_min AS "total_scores_per_min!",
    replays_per_min AS "replays_per_min!",
    perfect_combos_per_min AS "perfect_combos_per_min!",

    min_pp AS "min_pp!",
    max_pp AS "max_pp!",
    sum_pp AS "sum_pp!",

    sum_total_score AS "sum_total_score!",
    sum_classic_total_score AS "sum_classic_total_score!",
    sum_legacy_total_score AS "sum_legacy_total_score!",

    max_classic_total_score AS "max_classic_total_score!",
    max_legacy_total_score AS "max_legacy_total_score!",

    sum_accuracy AS "sum_accuracy!",
    peak_combo AS "peak_combo!"
FROM scores_per_minute
WHERE bucket >= NOW() - INTERVAL '24 hours'
ORDER BY bucket ASC;
            "#
        )
        .fetch_all(&*self)
        .await?;

        Ok(data)
    }

    pub async fn get_latest_minute_historic_graph(&self) -> Result<ScorePerMinute> {
        tracing::debug!("Fetching latest minute historic graph");

        let data = query_as!(
            ScorePerMinute,
            r#"
SELECT
    COALESCE(EXTRACT(EPOCH FROM bucket)::BIGINT, 0) AS "bucket!",
    ruleset_id AS "ruleset_id!",
    client_type AS "client_type!",

    hll_cardinality(user_hll) AS "unique_user_count!",
    hll_cardinality(beatmap_hll) AS "unique_beatmap_count!",

    total_scores_per_min AS "total_scores_per_min!",
    replays_per_min AS "replays_per_min!",
    perfect_combos_per_min AS "perfect_combos_per_min!",

    min_pp AS "min_pp!",
    max_pp AS "max_pp!",
    sum_pp AS "sum_pp!",

    sum_total_score AS "sum_total_score!",
    sum_classic_total_score AS "sum_classic_total_score!",
    sum_legacy_total_score AS "sum_legacy_total_score!",

    max_classic_total_score AS "max_classic_total_score!",
    max_legacy_total_score AS "max_legacy_total_score!",

    sum_accuracy AS "sum_accuracy!",
    peak_combo AS "peak_combo!"
FROM scores_per_minute
WHERE bucket >= NOW() - INTERVAL '24 hours'
ORDER BY bucket DESC
LIMIT 1;
    "#
        )
        .fetch_one(&*self)
        .await?;

        Ok(data)
    }
}
