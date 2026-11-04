use sqlx::prelude::FromRow;

mod impls;

#[derive(Debug, FromRow)]
pub struct DailyEntry {
    pub date: i64,
    pub stable: i64,
    pub lazer: i64,
}

#[derive(Debug)]
pub struct RatioRegression {
    pub target_ratio: f64,
    pub was_reached: bool,
    pub estimated_timestamp: i64,
}

/// Lazer share of the combined stable + lazer user count.
pub fn ratio(stable: i64, lazer: i64) -> f64 {
    let (stable, lazer) = (stable as f64, lazer as f64);
    lazer / (stable + lazer)
}
