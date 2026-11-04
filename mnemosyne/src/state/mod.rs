use crate::api::{
    PointLineResponse, ScoreAggregateResponse, SinglePointResponse, UserIdDistributionEntry,
};
use crate::database::{
    Database, Timestamp,
    scores::{BucketSize, LatestScore, ScoreDistributionResponse},
    scores_per_minute::ScorePerMinute,
};
use futures::TryFutureExt;
use mnemosyne_macros::valkey_cache;

pub struct State {
    cache: redis::aio::ConnectionManager,
    database: Database,
}

impl State {
    pub async fn new(
        database_url: &str,
        cache_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::new(database_url)
            .await
            .map_err(|e| format!("Failed to initialize database: {e}"))?;

        let cache = redis::Client::open(cache_url)
            .map_err(|e| format!("Failed to initialize cache connection: {e}"))?;

        let connection_manager = cache
            .get_connection_manager()
            .await
            .map_err(|e| format!("Failed to initialize connection manager: {e}"))?;

        Ok(Self {
            cache: connection_manager,
            database: db,
        })
    }

    pub fn database(&self) -> &Database {
        &self.database
    }
}

fn next_1am_from(now: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
    (now + chrono::TimeDelta::days(1))
        .date_naive()
        .and_hms_opt(1, 0, 0)
        .expect("1am is a valid time")
        .and_utc()
}

fn seconds_until_next_1am() -> u64 {
    let now = chrono::Utc::now();
    (next_1am_from(now) - now).num_seconds().max(1) as u64
}

fn aggregate_bucket(bucket_size: &BucketSize) -> &'static str {
    match bucket_size {
        BucketSize::Day => "daily",
        BucketSize::Week => "weekly",
        BucketSize::Month => "monthly",
    }
}

fn aggregate_ttl(bucket_size: &BucketSize) -> u64 {
    match bucket_size {
        BucketSize::Month => 604800,
        BucketSize::Day | BucketSize::Week => 86400,
    }
}

fn complete_history_key(bucket_size: &BucketSize) -> String {
    match bucket_size {
        BucketSize::Day => "athena:changelogs:graph:history:complete".to_owned(),
        BucketSize::Week => "athena:changelogs:graph:history:complete:weekly".to_owned(),
        BucketSize::Month => "athena:changelogs:graph:history:complete:monthly".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::next_1am_from;

    #[test]
    fn next_1am_is_next_calendar_day() {
        let now = Utc.timestamp_opt(1782675578, 0).unwrap();

        assert_eq!(
            next_1am_from(now),
            Utc.with_ymd_and_hms(2026, 6, 29, 1, 0, 0).unwrap()
        );
    }
}

valkey_cache! {
    State,
    measurements_history_range: PointLineResponse {
        key: format!("measurements:history_range:{}:{}", start.timestamp_micros(), end.timestamp_micros()),
        args: (start: Timestamp, end: Timestamp),
        ttl: 60,
        refresh: || self.database.get_history_range(start, end),
    },
}

valkey_cache! {
    State,
    scores_last_inserted: Option<i64> {
        key: "scores:last_inserted",
        ttl: 60,
        refresh: || self.database.get_last_inserted_score(),
    },
    scores_latest_inserted: LatestScore {
        key: "scores:latest_inserted",
        ttl: 60,
        refresh: || self.database.get_latest_inserted_score(),
    },
    scores_distribution_in_range: ScoreDistributionResponse {
        key: format!("scores:distribution_in_range:{}:{}", from.timestamp_micros(), to.timestamp_micros()),
        args: (from: Timestamp, to: Timestamp),
        ttl: 60,
        refresh: || self.database.get_score_distribution_in_range(from, to),
    },
    scores_per_minute_graphs: Vec<ScorePerMinute> {
        key: "scores_per_minute:graphs",
        ttl: 60,
        refresh: || self.database.get_minute_historic_graphs(),
    },
    scores_per_minute_recent_graphs: Vec<ScorePerMinute> {
        key: "scores_per_minute:recent_graphs",
        ttl: 60,
        refresh: || self.database.get_recent_minute_historic_graphs(),
    },
    scores_per_minute_latest_graph: ScorePerMinute {
        key: "scores_per_minute:latest_graph",
        ttl: 60,
        refresh: || self.database.get_latest_minute_historic_graph(),
    },
}

// Cache entries consumed by Athena. Keys and TTLs are kept compatible with
// Athena's previous local cache layer.
valkey_cache! {
    State,
    latest_changelog: SinglePointResponse {
        key: "athena:changelogs:changelog:latest",
        ttl: 60,
        refresh: || self.database.get_latest(),
    },
    peak_user_count: SinglePointResponse {
        key: "athena:changelogs:peak:users",
        ttl: 300,
        refresh: || self.database.get_user_count_peak(),
    },
    peak_user_ratio: SinglePointResponse {
        key: "athena:changelogs:peak:ratio",
        ttl: 300,
        refresh: || self.database.get_user_ratio_peak(),
    },
    day_user_graph: PointLineResponse {
        key: "athena:changelogs:graph:day",
        ttl: 300,
        refresh: || self.database.get_past_day(),
    },
    both_clients_history_graph: PointLineResponse {
        key: "athena:changelogs:graph:history",
        ttl: 300,
        refresh: || self.database.get_lazer_history(BucketSize::Day),
    },
    complete_history_graph: PointLineResponse {
        key: complete_history_key(&bucket_size),
        args: (bucket_size: BucketSize),
        ttl: 86400,
        refresh: || self.database.get_complete_history(bucket_size),
    },
    unique_users_by_id: Vec<UserIdDistributionEntry> {
        key: format!("athena:unique_users_by_id:{}", aggregate_bucket(&bucket_range)),
        args: (bucket_range: BucketSize),
        ttl: aggregate_ttl(&bucket_range),
        refresh: || self.database.get_unique_buckets(bucket_range),
        map: |v| v.into_iter().map(UserIdDistributionEntry::from).collect(),
    },
    scores_by_id: Vec<UserIdDistributionEntry> {
        key: format!("athena:unique_scores:{}", aggregate_bucket(&bucket_range)),
        args: (bucket_range: BucketSize),
        ttl: aggregate_ttl(&bucket_range),
        refresh: || self.database.get_bucketed_scores(bucket_range),
        map: |v| v.into_iter().map(UserIdDistributionEntry::from).collect(),
    },
    daily_historic_graphs: Vec<ScoreAggregateResponse> {
        key: "athena:daily_graph",
        ttl: seconds_until_next_1am(),
        refresh: || self.database.get_daily_historic_graphs(),
        map: |v| v.into_iter().map(ScoreAggregateResponse::from).collect(),
    },
}
