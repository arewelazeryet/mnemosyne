use bigdecimal::ToPrimitive as _;
use serde::{Deserialize, Serialize};

use crate::database::{
    changelog_counts_daily_aggregate::{DailyEntry, RatioRegression, ratio},
    measurements::MeasurementEntry,
    scores::{BucketedResponse, ClientType, ScoreGameMode},
    scores_daily_historic::ScoresAggregate,
};

/// Changelog API entry, simplified
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SinglePointResponse {
    pub timestamp: i64,
    pub stable: i64,
    pub lazer: i64,
    pub sum: i64,
    pub ratio: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PointLineResponse {
    pub timestamp: Vec<i64>,
    pub stable: Vec<i64>,
    pub lazer: Vec<i64>,
    pub sum: Vec<i64>,
    pub ratio: Vec<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserIdDistributionEntry {
    pub stable: u32,
    pub lazer: u32,
    pub both: u32,
    pub bucket: u32,
}

impl From<BucketedResponse> for UserIdDistributionEntry {
    fn from(value: BucketedResponse) -> Self {
        Self {
            stable: value.stable as u32,
            lazer: value.lazer as u32,
            both: value.both as u32,
            bucket: value.bucket_floor as u32,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct RatioRegressionResponse {
    pub target_ratio: f64,
    pub was_reached: bool,
    pub estimated_timestamp: i64,
}

impl From<MeasurementEntry> for SinglePointResponse {
    fn from(value: MeasurementEntry) -> Self {
        Self {
            timestamp: value.timestamp,
            stable: value.stable,
            lazer: value.lazer,
            ratio: ratio(value.stable, value.lazer),
            sum: value.stable + value.lazer,
        }
    }
}

impl From<SinglePointResponse> for MeasurementEntry {
    fn from(value: SinglePointResponse) -> Self {
        Self {
            timestamp: value.timestamp,
            stable: value.stable,
            lazer: value.lazer,
        }
    }
}

impl From<Vec<MeasurementEntry>> for PointLineResponse {
    fn from(value: Vec<MeasurementEntry>) -> Self {
        let mut response = Self {
            timestamp: Vec::with_capacity(value.len()),
            stable: Vec::with_capacity(value.len()),
            lazer: Vec::with_capacity(value.len()),
            sum: Vec::with_capacity(value.len()),
            ratio: Vec::with_capacity(value.len()),
        };

        for entry in value {
            response.timestamp.push(entry.timestamp);
            response.stable.push(entry.stable);
            response.lazer.push(entry.lazer);
            response.sum.push(entry.stable + entry.lazer);
            response.ratio.push(ratio(entry.stable, entry.lazer));
        }

        response
    }
}

impl From<Vec<DailyEntry>> for PointLineResponse {
    fn from(value: Vec<DailyEntry>) -> Self {
        let mut response = Self {
            timestamp: Vec::with_capacity(value.len()),
            stable: Vec::with_capacity(value.len()),
            lazer: Vec::with_capacity(value.len()),
            sum: Vec::with_capacity(value.len()),
            ratio: Vec::with_capacity(value.len()),
        };
        for entry in value {
            response.timestamp.push(entry.date);
            response.stable.push(entry.stable);
            response.lazer.push(entry.lazer);
            response.sum.push(entry.stable + entry.lazer);
            response.ratio.push(ratio(entry.stable, entry.lazer));
        }
        response
    }
}

impl From<RatioRegression> for RatioRegressionResponse {
    fn from(value: RatioRegression) -> Self {
        Self {
            target_ratio: value.target_ratio,
            was_reached: value.was_reached,
            estimated_timestamp: value.estimated_timestamp,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreAggregateResponse {
    pub day_bucket: i64,
    pub ruleset_id: ScoreGameMode,
    pub client_type: ClientType,
    pub unique_user_count: i64,
    pub unique_beatmap_count: i64,
    pub total_daily_scores: u32,
    pub daily_scores_with_replays: u32,
    pub daily_perfect_combos: u32,
    pub daily_min_pp: f64,
    pub daily_max_pp: f64,
    pub daily_sum_pp: f64,
    pub daily_average_pp: f32,
    pub daily_sum_total_score: bigdecimal::BigDecimal,
    pub daily_sum_classic_total_score: bigdecimal::BigDecimal,
    pub daily_sum_legacy_total_score: bigdecimal::BigDecimal,
    pub daily_max_classic_total_score: i64,
    pub daily_max_legacy_total_score: i64,
    pub daily_average_accuracy: f32,
    pub daily_peak_combo: i32,
}

impl From<&ScoresAggregate> for ScoreAggregateResponse {
    fn from(value: &ScoresAggregate) -> Self {
        ScoreAggregateResponse {
            day_bucket: value.day_bucket,
            ruleset_id: value.ruleset_id,
            client_type: value.client_type,
            unique_user_count: value.unique_user_count as i64,
            unique_beatmap_count: value.unique_beatmap_count as i64,
            total_daily_scores: value
                .total_daily_scores
                .clone()
                .to_u32()
                .unwrap_or_default(),
            daily_scores_with_replays: value.daily_scores_with_replays.to_u32().unwrap_or_default(),
            daily_perfect_combos: value.daily_perfect_combos.to_u32().unwrap_or_default(),
            daily_min_pp: value.daily_min_pp,
            daily_max_pp: value.daily_max_pp,
            daily_sum_pp: value.daily_sum_pp,
            daily_sum_total_score: value.daily_sum_total_score.clone(),
            daily_sum_classic_total_score: value.daily_sum_classic_total_score.clone(),
            daily_sum_legacy_total_score: value.daily_sum_legacy_total_score.clone(),
            daily_max_classic_total_score: value.daily_max_classic_total_score,
            daily_max_legacy_total_score: value.daily_max_legacy_total_score,
            daily_average_accuracy: (value.daily_sum_accuracy / value.total_daily_scores.clone())
                .to_f32()
                .unwrap_or_default(),
            daily_average_pp: (value.daily_sum_pp / value.total_daily_scores.clone())
                .to_f32()
                .unwrap_or_default(),
            daily_peak_combo: value.daily_peak_combo,
        }
    }
}

impl From<ScoresAggregate> for ScoreAggregateResponse {
    fn from(value: ScoresAggregate) -> Self {
        ScoreAggregateResponse {
            day_bucket: value.day_bucket,
            ruleset_id: value.ruleset_id,
            client_type: value.client_type,
            unique_user_count: value.unique_user_count as i64,
            unique_beatmap_count: value.unique_beatmap_count as i64,
            total_daily_scores: value
                .total_daily_scores
                .clone()
                .to_u32()
                .unwrap_or_default(),
            daily_scores_with_replays: value.daily_scores_with_replays.to_u32().unwrap_or_default(),
            daily_perfect_combos: value.daily_perfect_combos.to_u32().unwrap_or_default(),
            daily_min_pp: value.daily_min_pp,
            daily_max_pp: value.daily_max_pp,
            daily_sum_pp: value.daily_sum_pp,
            daily_sum_total_score: value.daily_sum_total_score.clone(),
            daily_sum_classic_total_score: value.daily_sum_classic_total_score.clone(),
            daily_sum_legacy_total_score: value.daily_sum_legacy_total_score.clone(),
            daily_max_classic_total_score: value.daily_max_classic_total_score,
            daily_max_legacy_total_score: value.daily_max_legacy_total_score,
            daily_average_accuracy: (value.daily_sum_accuracy / value.total_daily_scores.clone())
                .to_f32()
                .unwrap_or_default(),
            daily_average_pp: (value.daily_sum_pp / value.total_daily_scores.clone())
                .to_f32()
                .unwrap_or_default(),
            daily_peak_combo: value.daily_peak_combo,
        }
    }
}
