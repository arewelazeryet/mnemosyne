use color_eyre::{Result, eyre::Context};

use crate::database::{Database, Timestamp, performance_rankings::PerformanceRuleset};

use super::PerformanceRankingRow;

impl Database {
    /// Upserts one fetch snapshot (all rows share `fetched_at`) in a single transaction.
    pub async fn insert_performance_rankings(&self, rows: &[PerformanceRankingRow]) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }

        let mut trans = self.begin().await?;
        for row in rows {
            sqlx::query!(
                r#"INSERT INTO pp_rankings (
                    fetched_at, user_id, user_name, country_code, accuracy,
                    count_300, count_100, count_50, count_miss,
                    ss_count, ssh_count, s_count, sh_count, a_count,
                    max_combo, playcount, playtime, ranked_score,
                    replays_watched, total_hits, total_score,
                    mode, rank, pp
                )
                VALUES (
                    $1, $2, $3, $4, $5,
                    $6, $7, $8, $9,
                    $10, $11, $12, $13, $14,
                    $15, $16, $17, $18,
                    $19, $20, $21,
                    $22, $23, $24
                )"#,
                row.fetched_at,
                row.user_id,
                row.user_name,
                row.country_code,
                row.accuracy,
                row.count_300,
                row.count_100,
                row.count_50,
                row.count_miss,
                row.ss_count,
                row.ssh_count,
                row.s_count,
                row.sh_count,
                row.a_count,
                row.max_combo,
                row.playcount,
                row.playtime,
                row.ranked_score,
                row.replays_watched,
                row.total_hits,
                row.total_score,
                row.mode as PerformanceRuleset,
                row.rank,
                row.pp,
            )
            .execute(&mut *trans)
            .await
            .wrap_err("Failed to insert performance ranking")?;
        }

        trans.commit().await?;

        Ok(())
    }
}
