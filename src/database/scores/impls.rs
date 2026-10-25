use std::time::{SystemTime, UNIX_EPOCH};

use color_eyre::{Result, eyre::Context};
use metrics::{counter, gauge};
use sqlx::{Postgres, QueryBuilder, query, query_as, query_scalar};

use crate::database::{Database, Timestamp};

use super::{BucketSize, BucketedResponse, DatabaseScore, LatestScore, ScoreDistributionResponse};

impl Database {
    /// Inserts a batch of scores in a single transaction.
    pub async fn insert_score(
        &self,
        scores: impl ExactSizeIterator<Item = &DatabaseScore>,
    ) -> Result<()> {
        let batch_length = scores.len();
        let span = tracing::info_span!(
            target: "insert_score_batch",
            "Score insertion",
            batch_size = batch_length
        );
        tracing::info!(parent: &span, "Inserting a new score batch");

        let mut trans = self.begin().await?;

        for score in scores {
            query!(
                r#"
    INSERT INTO scores
    (id, user_id, ruleset_id, beatmap_id, has_replay, grade, accuracy, max_combo, total_score, classic_total_score, total_score_without_mods, is_perfect_combo, legacy_perfect, pp, legacy_total_score, ended_at, build_id, lazer, data)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                "#,
                score.id as i64,
                score.user_id as i64,
                score.mode as i16,
                score.beatmap_id as i64,
                score.replay,
                score.grade.to_string(),
                score.accuracy,
                score.max_combo as i32,
                score.total_score as i64,
                score.classic_total_score as i64,
                score.total_score_without_mods.map(|s| s as i64),
                score.is_perfect_combo,
                score.legacy_perfect,
                score.pp as f64,
                score.legacy_total_score as i64,
                score.ended_at,
                score.build_id as i16,
                score.lazer,
                serde_json::to_value(&score.data)?,
            )
            .execute(&mut *trans)
            .await
            .wrap_err("Failed to insert a result")?;
        }

        trans.commit().await?;

        counter!(
            description: "Number of scores inserted",
            unit: metrics::Unit::Count,
            "nike.scores_inserted_since_startup"
        )
        .increment(batch_length as u64);

        gauge!(
            description: "Timestamp of the last score insertion",
            unit: metrics::Unit::Seconds,
            "nike.last_insertion_success_seconds"
        )
        .set(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
        );

        tracing::info!(parent: &span, "Commit: inserted {batch_length} scores");

        Ok(())
    }

    /// Newest score id within the last 72 hours, used to resume the score feed.
    pub async fn get_last_inserted_score(&self) -> Result<Option<i64>> {
        query_scalar::<_, i64>(
            r#"SELECT id FROM scores
            WHERE ended_at >= NOW() - INTERVAL '72 hours'
            ORDER BY id DESC LIMIT 1"#,
        )
        .fetch_optional(&*self)
        .await
        .wrap_err("Failed to fetch last score")
    }

    /// Newest score (id and timestamp) within the last 12 hours.
    pub async fn get_latest_inserted_score(&self) -> Result<LatestScore> {
        query_as!(
            LatestScore,
            r#"SELECT id, ended_at FROM scores
            WHERE ended_at >= NOW() - INTERVAL '12 hours'
            ORDER BY id DESC LIMIT 1"#,
        )
        .fetch_one(&*self)
        .await
        .wrap_err("Failed to fetch last score")
    }

    /// Fetches the score distribution for all scores in the timestamp range.
    pub async fn get_score_distribution_in_range(
        &self,
        from: Timestamp,
        to: Timestamp,
    ) -> Result<ScoreDistributionResponse> {
        query_as!(
            ScoreDistributionResponse,
            r#"
            SELECT
                COUNT(*) FILTER (WHERE lazer = true)  AS "lazer!",
                COUNT(*) FILTER (WHERE lazer = false) AS "stable!"
            FROM scores
            WHERE ended_at > $1 AND ended_at < $2
            "#,
            from,
            to,
        )
        .fetch_one(&*self)
        .await
        .wrap_err("Failed to fetch score distribution")
    }

    pub async fn get_unique_buckets(
        &self,
        bucket_range: BucketSize,
    ) -> Result<Vec<BucketedResponse>> {
        tracing::info!(time_range = ?bucket_range, "Getting unique buckets");
        counter!(format!(
            "athena.database.get_{}_unique_users.query_count",
            bucket_range
        ))
        .increment(1);

        let mut trans = self.begin_with_chunkwise_aggregation_disabled().await?;

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                (user_id / 2000000) * 2000000 AS "bucket_floor",
                COUNT(*) FILTER (WHERE has_stable AND NOT has_lazer) AS "stable",
                COUNT(*) FILTER (WHERE has_lazer AND NOT has_stable) AS "lazer",
                COUNT(*) FILTER (WHERE has_lazer AND has_stable) AS "both"
            FROM (
                SELECT
                    user_id,
                    BOOL_OR(lazer) AS has_lazer,
                    BOOL_OR(NOT lazer) AS has_stable
                FROM scores
                WHERE ended_at >= NOW() - INTERVAL "#,
        );
        builder.push(bucket_range.as_sql_interval());
        builder.push(
            r#"
                GROUP BY user_id
            ) u
            GROUP BY "bucket_floor"
            ORDER BY "bucket_floor" ASC;
        "#,
        );

        let result = builder
            .build_query_as::<BucketedResponse>()
            .fetch_all(&mut *trans)
            .await
            .wrap_err("Failed to get monthly unique users");

        trans.commit().await?;

        tracing::info!(time_range = ?bucket_range, "Got unique buckets");

        result
    }

    pub async fn get_bucketed_scores(
        &self,
        bucket_range: BucketSize,
    ) -> Result<Vec<BucketedResponse>> {
        tracing::info!(time_range = ?bucket_range, "Getting unique buckets");
        counter!(format!(
            "athena.database.get_{}_unique_scores.query_count",
            bucket_range
        ))
        .increment(1);

        let mut trans = self.begin_with_chunkwise_aggregation_disabled().await?;

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                ((user_id / 2000000) * 2000000)::INT8 AS "bucket_floor",
                (COUNT(*) FILTER (WHERE lazer))::INT8 AS "lazer",
                (COUNT(*) FILTER (WHERE NOT lazer))::INT8 AS "stable",
                0::INT8 AS "both"
            FROM scores
            WHERE ended_at >= NOW() - INTERVAL "#,
        );
        builder.push(bucket_range.as_sql_interval());
        builder.push(
            r#"
            GROUP BY "bucket_floor"
            ORDER BY "bucket_floor" ASC;
        "#,
        );

        let result = builder
            .build_query_as::<BucketedResponse>()
            .fetch_all(&mut *trans)
            .await
            .wrap_err("Failed to get monthly unique users");

        trans.commit().await?;

        tracing::info!(time_range = ?bucket_range, "Got unique buckets");

        result
    }
}
