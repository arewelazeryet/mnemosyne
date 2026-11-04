use std::fmt;

use chrono::{DateTime, Utc};
use rosu_v2::model::{GameMode, Grade, score::Score};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::database::Timestamp;

mod impls;

/// Trimmed score information for the database.
#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct DatabaseScore {
    pub id: u64,
    pub user_id: u32,
    pub mode: GameMode,
    pub beatmap_id: u32,
    pub replay: bool,
    pub grade: Grade,
    pub accuracy: f32,
    pub max_combo: u32,
    pub build_id: u8,
    pub total_score: u32,
    pub classic_total_score: u64,
    pub legacy_total_score: u32,
    pub total_score_without_mods: Option<u32>,
    pub map_id: u32,
    pub ended_at: Timestamp,
    pub pp: f32,
    pub is_perfect_combo: bool,
    pub legacy_perfect: Option<bool>,
    pub lazer: bool,
    pub data: Data,
}

impl From<Score> for DatabaseScore {
    fn from(value: Score) -> Self {
        Self {
            id: value.id as u64,
            user_id: value.user_id,
            build_id: value.build_id.unwrap_or_default() as u8,
            beatmap_id: value.map_id,
            total_score: value.score,
            classic_total_score: value.classic_score,
            legacy_total_score: value.legacy_score,
            total_score_without_mods: value.total_score_without_mods,
            map_id: value.map_id,
            accuracy: value.accuracy,
            max_combo: value.max_combo,
            ended_at: from_unix_nanos(value.ended_at.unix_timestamp_nanos()),
            grade: value.grade,
            pp: value.pp.unwrap_or_default(),
            replay: value.replay,
            legacy_perfect: value.legacy_perfect,
            is_perfect_combo: value.is_perfect_combo,
            mode: value.mode,
            lazer: value.set_on_lazer,
            data: Data {
                max_score_statistics: value.maximum_statistics.into(),
                score_statistics: value.statistics.into(),
                mods: serde_json::value::to_raw_value(&value.mods)
                    .expect("score mods are serializable"),
            },
        }
    }
}

/// Converts Unix nanoseconds into a UTC timestamp.
///
/// # Panics
///
/// Panics if the value is outside `chrono`'s timestamp range.
fn from_unix_nanos(nanos: i128) -> Timestamp {
    let seconds = i64::try_from(nanos.div_euclid(1_000_000_000)).expect("timestamp out of range");
    let nanosecond = nanos.rem_euclid(1_000_000_000) as u32;

    DateTime::<Utc>::from_timestamp(seconds, nanosecond).expect("timestamp out of range")
}

/// Extra data (max stats, stats, mods).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Data {
    pub max_score_statistics: ScoreStatistics,
    pub score_statistics: ScoreStatistics,
    pub mods: Box<serde_json::value::RawValue>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScoreStatistics {
    #[serde(alias = "count_miss", skip_serializing_if = "Option::is_none")]
    pub miss: Option<u32>,
    #[serde(alias = "count_50", skip_serializing_if = "Option::is_none")]
    pub meh: Option<u32>,
    #[serde(alias = "count_100", skip_serializing_if = "Option::is_none")]
    pub ok: Option<u32>,
    #[serde(alias = "count_katu", skip_serializing_if = "Option::is_none")]
    pub good: Option<u32>,
    #[serde(alias = "count_300", skip_serializing_if = "Option::is_none")]
    pub great: Option<u32>,
    #[serde(alias = "count_geki", skip_serializing_if = "Option::is_none")]
    pub perfect: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_tick_hit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_tick_miss: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_tick_hit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_tick_miss: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_hit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_miss: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_bonus: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_bonus: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slider_tail_hit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combo_break: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_combo_increase: Option<u32>,
}

macro_rules! none_if_default {
    ($value:expr) => {
        if $value == u32::default() {
            None
        } else {
            Some($value)
        }
    };
}

impl From<rosu_v2::model::score::ScoreStatistics> for ScoreStatistics {
    fn from(value: rosu_v2::model::score::ScoreStatistics) -> Self {
        ScoreStatistics {
            miss: none_if_default!(value.miss),
            meh: none_if_default!(value.meh),
            ok: none_if_default!(value.ok),
            good: none_if_default!(value.good),
            great: none_if_default!(value.great),
            perfect: none_if_default!(value.perfect),
            large_tick_hit: none_if_default!(value.large_tick_hit),
            large_tick_miss: none_if_default!(value.large_tick_miss),
            small_tick_hit: none_if_default!(value.small_tick_hit),
            small_tick_miss: none_if_default!(value.small_tick_miss),
            ignore_hit: none_if_default!(value.ignore_hit),
            ignore_miss: none_if_default!(value.ignore_miss),
            large_bonus: none_if_default!(value.large_bonus),
            small_bonus: none_if_default!(value.small_bonus),
            slider_tail_hit: none_if_default!(value.slider_tail_hit),
            combo_break: none_if_default!(value.combo_break),
            legacy_combo_increase: none_if_default!(value.legacy_combo_increase),
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, sqlx::Type, Clone, Copy)]
#[sqlx(type_name = "ClientType")]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
    Stable = 0,
    #[default]
    Lazer = 1,
}

impl From<String> for ClientType {
    fn from(value: String) -> Self {
        match value.as_ref() {
            "stable" => ClientType::Stable,
            "lazer" => ClientType::Lazer,
            _ => unreachable!(),
        }
    }
}

impl From<i32> for ClientType {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Stable,
            1 => Self::Lazer,
            _ => unreachable!(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScoreGameMode {
    /// osu!standard
    #[default]
    Osu = 0,
    /// osu!taiko
    Taiko = 1,
    /// osu!catch
    Catch = 2,
    /// osu!mania
    Mania = 3,
}

impl From<i16> for ScoreGameMode {
    fn from(value: i16) -> Self {
        match value {
            0 => Self::Osu,
            1 => Self::Taiko,
            2 => Self::Catch,
            3 => Self::Mania,
            _ => unreachable!(),
        }
    }
}

/// Time window used by the bucketed score queries and the changelog history.
#[derive(Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub enum BucketSize {
    #[default]
    Day,
    Week,
    Month,
}

impl BucketSize {
    pub(crate) const fn as_sql_interval(&self) -> &'static str {
        match self {
            Self::Day => "'1 day'",
            Self::Week => "'7 days'",
            Self::Month => "'30 days'",
        }
    }
}

impl fmt::Display for BucketSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
        };

        f.write_str(name)
    }
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct ScoreDistributionResponse {
    pub stable: i64,
    pub lazer: i64,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct BucketedResponse {
    pub bucket_floor: i64,
    pub stable: i64,
    pub lazer: i64,
    pub both: i64,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct LatestScore {
    pub ended_at: Timestamp,
    pub id: i64,
}
