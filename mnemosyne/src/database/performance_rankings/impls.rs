use color_eyre::{Result, eyre::Context};

use crate::database::{Database, Timestamp, performance_rankings::PerformanceRuleset};

use super::PerformanceRankingRow;

impl Database {
    /// Upserts one fetch snapshot (all rows share `fetched_at`) in a single transaction.
    pub async fn insert_performance_rankings(
        &self,
        rows: &[PerformanceRankingRow],
    ) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }

        let mut trans = self.begin().await?;

        for row in rows {
            sqlx::query!(
                r#"INSERT INTO pp_rankings (fetched_at, user_id, user_name, mode, rank, pp, country_code)
                VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
                row.fetched_at,
                row.user_id,
                row.user_name,
                row.mode as PerformanceRuleset,
                row.rank,
                row.pp,
                row.country_code,
            )
            .execute(&mut *trans)
            .await
            .wrap_err("Failed to insert performance ranking")?;
        }

        trans.commit().await?;

        Ok(())
    }
}
