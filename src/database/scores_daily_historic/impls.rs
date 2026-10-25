use color_eyre::Result;
use sqlx::query_as;

use crate::database::Database;

use super::ScoresAggregate;

impl Database {
    pub async fn get_daily_historic_graphs(&self) -> Result<Vec<ScoresAggregate>> {
        tracing::debug!("Fetching daily historic graphs");

        let aggregates = query_as!(
            ScoresAggregate,
            r#"
SELECT
    COALESCE(EXTRACT(EPOCH FROM day_bucket)::BIGINT, 0) AS "day_bucket!",
    ruleset_id AS "ruleset_id!",
    client_type AS "client_type!",

    hll_cardinality(user_hll) AS "unique_user_count!",
    hll_cardinality(beatmap_hll) AS "unique_beatmap_count!",

    total_daily_scores AS "total_daily_scores!",
    daily_scores_with_replays AS "daily_scores_with_replays!",
    daily_perfect_combos AS "daily_perfect_combos!",

    daily_min_pp AS "daily_min_pp!",
    daily_max_pp AS "daily_max_pp!",
    daily_sum_pp AS "daily_sum_pp!",

    daily_sum_total_score AS "daily_sum_total_score!",
    daily_sum_classic_total_score AS "daily_sum_classic_total_score!",
    daily_sum_legacy_total_score AS "daily_sum_legacy_total_score!",

    daily_max_classic_total_score AS "daily_max_classic_total_score!",
    daily_max_legacy_total_score AS "daily_max_legacy_total_score!",

    daily_sum_accuracy AS "daily_sum_accuracy!",
    daily_peak_combo AS "daily_peak_combo!"
FROM scores_daily_historic;
            "#,
        )
        .fetch_all(&*self)
        .await?;

        Ok(aggregates)
    }
}
