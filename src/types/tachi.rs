use super::GameVersion;
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub meta: ImportMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<ImportClasses>,
    pub scores: Vec<ImportScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMeta {
    pub game: String,
    pub service: String,
    pub version: String,
}

impl ImportMeta {
    pub fn new(version: GameVersion) -> Self {
        Self {
            game: "sdvx".to_string(),
            service: "Mikado".to_string(),
            version: version.tachi_id().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportClasses {
    pub dan: SkillLevel,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, FromPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum SkillLevel {
    #[num_enum(default)]
    #[serde(rename = "DAN_1")]
    First = 1,
    #[serde(rename = "DAN_2")]
    Second = 2,
    #[serde(rename = "DAN_3")]
    Third = 3,
    #[serde(rename = "DAN_4")]
    Fourth = 4,
    #[serde(rename = "DAN_5")]
    Fifth = 5,
    #[serde(rename = "DAN_6")]
    Sixth = 6,
    #[serde(rename = "DAN_7")]
    Seventh = 7,
    #[serde(rename = "DAN_8")]
    Eighth = 8,
    #[serde(rename = "DAN_9")]
    Ninth = 9,
    #[serde(rename = "DAN_10")]
    Tenth = 10,
    #[serde(rename = "DAN_11")]
    Eleventh = 11,
    #[serde(rename = "INF")]
    Infinite = 12,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportScore {
    pub score: u32,
    pub lamp: TachiLamp,
    #[serde(rename = "matchType")]
    pub match_type: String,
    pub identifier: String,
    pub difficulty: TachiDifficulty,
    #[serde(rename = "timeAchieved")]
    pub time_achieved: u128,
    pub judgements: Judgements,
    #[serde(rename = "hitMeta")]
    pub hit_meta: HitMeta,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TachiLamp {
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "CLEAR")]
    Clear,
    #[serde(rename = "EXCESSIVE CLEAR")]
    ExcessiveClear,
    #[serde(rename = "ULTIMATE CHAIN")]
    UltimateChain,
    #[serde(rename = "PERFECT ULTIMATE CHAIN")]
    PerfectUltimateChain,
    #[serde(rename = "MAXXIVE CLEAR")]
    MaxxiveClear,
}

impl TachiLamp {
    pub fn from_clear_type(version: GameVersion, clear_type: u32) -> Self {
        match version {
            GameVersion::ExceedGear => match clear_type {
                2 => TachiLamp::Clear,
                3 => TachiLamp::ExcessiveClear,
                4 => TachiLamp::UltimateChain,
                5 => TachiLamp::PerfectUltimateChain,
                6 => TachiLamp::MaxxiveClear,
                _ => TachiLamp::Failed,
            },
            GameVersion::Nabla => match clear_type {
                2 => TachiLamp::Clear,
                3 => TachiLamp::ExcessiveClear,
                4 => TachiLamp::MaxxiveClear,
                5 => TachiLamp::UltimateChain,
                6 => TachiLamp::PerfectUltimateChain,
                _ => TachiLamp::Failed,
            },
        }
    }

    pub fn to_index(self, version: GameVersion) -> u32 {
        match version {
            GameVersion::ExceedGear => match self {
                TachiLamp::Failed => 1,
                TachiLamp::Clear => 2,
                TachiLamp::ExcessiveClear => 3,
                TachiLamp::UltimateChain => 4,
                TachiLamp::PerfectUltimateChain => 5,
                TachiLamp::MaxxiveClear => 6,
            },
            GameVersion::Nabla => match self {
                TachiLamp::Failed => 1,
                TachiLamp::Clear => 2,
                TachiLamp::ExcessiveClear => 3,
                TachiLamp::MaxxiveClear => 4,
                TachiLamp::UltimateChain => 5,
                TachiLamp::PerfectUltimateChain => 6,
            },
        }
    }
}

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Hash, FromPrimitive, IntoPrimitive, Serialize, Deserialize,
)]
#[repr(u32)]
pub enum TachiDifficulty {
    #[num_enum(default)]
    #[serde(rename = "NOV")]
    Novice = 0,
    #[serde(rename = "ADV")]
    Advanced = 1,
    #[serde(rename = "EXH")]
    Exhaust = 2,
    #[serde(rename = "ANY_INF")]
    AnyInfinite = 3,
    #[serde(rename = "MXM")]
    Maximum = 4,
    #[serde(rename = "ULT")]
    Ultimate = 5,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Judgements {
    pub critical: u32,
    pub near: u32,
    pub miss: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HitMeta {
    pub fast: u32,
    pub slow: u32,
    #[serde(rename = "maxCombo")]
    pub max_combo: u32,
    #[serde(rename = "exScore")]
    pub ex_score: Option<u32>,
    pub gauge: f32,
}
