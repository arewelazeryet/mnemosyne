use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::database::scores::{ClientType, ScoreGameMode};

mod impls;

#[derive(FromRow, Deserialize, Serialize)]
pub struct ScoresAggregate {
    pub day_bucket: i64,
    #[sqlx(try_from = "i16")]
    pub ruleset_id: ScoreGameMode,
    #[sqlx(try_from = "i32")]
    pub client_type: ClientType,
    pub unique_user_count: f64,
    pub unique_beatmap_count: f64,
    pub total_daily_scores: BigDecimal,
    pub daily_scores_with_replays: BigDecimal,
    pub daily_perfect_combos: BigDecimal,
    pub daily_min_pp: f64,
    pub daily_max_pp: f64,
    pub daily_sum_pp: f64,
    pub daily_sum_total_score: BigDecimal,
    pub daily_sum_classic_total_score: BigDecimal,
    pub daily_sum_legacy_total_score: BigDecimal,
    pub daily_max_classic_total_score: i64,
    pub daily_max_legacy_total_score: i64,
    pub daily_sum_accuracy: f64,
    pub daily_peak_combo: i32,
}
