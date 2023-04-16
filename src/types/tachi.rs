use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Import {
    pub meta: ImportMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classes: Option<ImportClasses>,
    pub scores: Vec<ImportScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMeta {
    pub game: String,
    #[serde(rename = "playtype")]
    pub play_type: String,
    pub service: String,
}

impl Default for ImportMeta {
    fn default() -> Self {
        Self {
            game: "sdvx".to_string(),
            play_type: "Single".to_string(),
            service: "Mikado".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportClasses {
    pub dan: SkillLevel,
}

#[derive(Debug, Clone, Eq, PartialEq, FromPrimitive, Serialize, Deserialize)]
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
    pub difficulty: Difficulty,
    #[serde(rename = "timeAchieved")]
    pub time_achieved: u128,
    pub judgements: Judgements,
    #[serde(rename = "hitMeta")]
    pub hit_meta: HitMeta,
}

#[derive(Debug, Clone, Eq, PartialEq, FromPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum TachiLamp {
    #[num_enum(default)]
    #[serde(rename = "FAILED")]
    Failed = 1,
    #[serde(rename = "CLEAR")]
    Clear = 2,
    #[serde(rename = "EXCESSIVE CLEAR")]
    ExcessiveClear = 3,
    #[serde(rename = "ULTIMATE CHAIN")]
    UltimateChain = 4,
    #[serde(rename = "PERFECT ULTIMATE CHAIN")]
    PerfectUltimateChain = 5,
}

#[derive(Debug, Clone, Eq, PartialEq, FromPrimitive, Serialize, Deserialize)]
#[repr(u32)]
pub enum Difficulty {
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
    pub ex_score: u32,
    pub gauge: f32,
}
