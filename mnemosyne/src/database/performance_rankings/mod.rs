mod impls;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::database::Timestamp;

/// One osu! performance-ranking entry to persist.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceRankingRow {
    pub fetched_at: Timestamp,
    pub user_id: i64,
    pub user_name: String,

    pub accuracy: f64,
    pub count_300: i32,
    pub count_100: i32,
    pub count_50: i32,
    pub count_miss: i32,
    pub ss_count: i32,
    pub ssh_count: i32,
    pub s_count: i32,
    pub sh_count: i32,
    pub a_count: i32,
    pub max_combo: i32,
    pub playcount: i32,
    pub playtime: i32,
    pub ranked_score: i64,
    pub replays_watched: i32,
    pub total_hits: i32,
    pub total_score: i64,

    pub mode: PerformanceRuleset,
    pub rank: i32,
    pub pp: f64,
    pub country_code: String,
}

#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, sqlx::Type, PartialEq, Eq, PartialOrd, Ord,
)]
#[sqlx(type_name = "pp_ruleset", rename_all = "lowercase")]
pub enum PerformanceRuleset {
    Osu,
    Taiko,
    Catch,
    ManiaAll,
    Mania4k,
    Mania7k,
}
