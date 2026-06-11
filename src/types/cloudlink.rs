use anyhow::Result;
use crate::{mikado};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Chart {
    pub song_id: u32,
    pub difficulty: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Score {
    property: [u32; 21],
    property_nabla: [u32; 26],
}

impl Score {
    pub fn from_cloud(score: u32, clear: u8, grade: u8) -> Self {
        let mut ret = Self::default();
        ret.property[17] = score;
        ret.property[18] = clear as u32;
        ret.property[19] = grade as u32;

        ret.property_nabla[18] = score;
        ret.property_nabla[20] = clear as u32;
        ret.property_nabla[21] = grade as u32;

        ret
    }

    pub fn from_slice(vec: &[u32]) -> Result<Self> {
        if vec.len() < 21 {
            return Err(anyhow::anyhow!("Could not parse score"));
        }
        let mut ret = Self::default();
        ret.property.copy_from_slice(&vec[..21]);
        if vec.len() >= 26 {
            ret.property_nabla.copy_from_slice(&vec[..26]);
        } else {
            // populate only the first 21 elements when full nabla properties aren't present
            ret.property_nabla[..21].copy_from_slice(&vec[..21]);
        }
        Ok(ret)
    }

    pub fn cloud_score_mut(&mut self) -> &mut u32 {
        if mikado::GAME_PROPERTIES.get().map(|p| p.is_nabla()).unwrap_or_default() {
            &mut self.property[18]
        } else {
            &mut self.property[17]
        }
    }

    pub fn cloud_clear_mut(&mut self) -> &mut u32 {
        if mikado::GAME_PROPERTIES.get().map(|p| p.is_nabla()).unwrap_or_default() {
            &mut self.property[20]
        } else {
            &mut self.property[18]
        }
    }

    pub fn cloud_grade_mut(&mut self) -> &mut u32 {
        if mikado::GAME_PROPERTIES.get().map(|p| p.is_nabla()).unwrap_or_default() {
            &mut self.property[21]
        } else {
            &mut self.property[19]
        }
    }

    pub fn to_property(self) -> Vec<u32> {
        if mikado::GAME_PROPERTIES.get().map(|p| p.is_nabla()).unwrap_or_default() {
            return self.property_nabla.to_vec();
        } else {
            return self.property.to_vec();
        }
    }
}
