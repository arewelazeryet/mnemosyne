use sqlx::prelude::FromRow;

mod impls;

#[derive(Debug, FromRow)]
pub struct MeasurementEntry {
    pub timestamp: i64,
    pub stable: i64,
    pub lazer: i64,
}
