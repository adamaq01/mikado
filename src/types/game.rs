use either::Either;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub call: CallStruct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStruct {
    #[serde(with = "either::serde_untagged")]
    pub game: Either<GameScores, GameSave>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameScores {
    #[serde(rename = "refid")]
    pub ref_id: String,
    #[serde(with = "either::serde_untagged", rename = "track")]
    pub tracks: Either<Track, Vec<Track>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Track {
    pub music_id: u32,
    pub music_type: u32,
    pub score: u32,
    #[serde(rename = "exscore")]
    pub ex_score: u32,
    pub clear_type: u32,
    pub max_chain: u32,
    pub critical: u32,
    pub near: u32,
    pub error: u32,
    pub effective_rate: u32,
    pub gauge_type: u32,
    pub judge: [u32; 7],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameSave {
    #[serde(rename = "refid")]
    pub ref_id: String,
    pub skill_level: u32,
}
