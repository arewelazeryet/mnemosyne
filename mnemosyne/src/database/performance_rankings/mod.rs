mod impls;

use serde::{Deserialize, Serialize};

use crate::database::Timestamp;

/// One osu! performance-ranking entry to persist.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceRankingRow {
    pub fetched_at: Timestamp,
    pub user_id: i64,
    pub user_name: String,
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
