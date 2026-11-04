use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::database::scores::{ClientType, ScoreGameMode};

mod impls;

#[derive(Debug, Deserialize, Serialize)]
pub struct ScorePerMinute {
    pub bucket: i64,
    pub ruleset_id: ScoreGameMode,
    pub client_type: ClientType,
    pub unique_user_count: f64,
    pub unique_beatmap_count: f64,
    pub total_scores_per_min: i64,
    pub replays_per_min: i64,
    pub perfect_combos_per_min: i64,
    pub min_pp: f64,
    pub max_pp: f64,
    pub sum_pp: f64,
    pub sum_total_score: BigDecimal,
    pub sum_classic_total_score: BigDecimal,
    pub sum_legacy_total_score: BigDecimal,
    pub max_classic_total_score: i64,
    pub max_legacy_total_score: i64,
    pub sum_accuracy: f64,
    pub peak_combo: i32,
}
