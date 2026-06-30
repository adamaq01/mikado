use anyhow::Result;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Chart {
    pub song_id: u32,
    pub difficulty: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum Score {
    ExceedGear([u32; 21]),
    Nabla([u32; 26]),
}

impl Score {
    pub fn from_cloud(is_nabla: bool, score: u32, clear: u8, grade: u8, ex_score: u32) -> Self {
        if is_nabla {
            let mut arr = [0u32; 26];
            arr[18] = score;
            arr[19] = ex_score;
            arr[20] = clear as u32;
            arr[21] = grade as u32;
            Score::Nabla(arr)
        } else {
            let mut arr = [0u32; 21];
            arr[17] = score;
            arr[18] = clear as u32;
            arr[19] = grade as u32;
            Score::ExceedGear(arr)
        }
    }

    pub fn from_slice(is_nabla: bool, vec: &[u32]) -> Result<Self> {
        if is_nabla {
            if vec.len() < 26 {
                return Err(anyhow::anyhow!("Could not parse score"));
            }
            let mut arr = [0u32; 26];
            arr.copy_from_slice(&vec[..26]);
            Ok(Score::Nabla(arr))
        } else {
            if vec.len() < 21 {
                return Err(anyhow::anyhow!("Could not parse score"));
            }
            let mut arr = [0u32; 21];
            arr.copy_from_slice(&vec[..21]);
            Ok(Score::ExceedGear(arr))
        }
    }

    pub fn cloud_score_mut(&mut self) -> &mut u32 {
        match self {
            Score::ExceedGear(arr) => &mut arr[17],
            Score::Nabla(arr) => &mut arr[18],
        }
    }

    pub fn cloud_clear_mut(&mut self) -> &mut u32 {
        match self {
            Score::ExceedGear(arr) => &mut arr[18],
            Score::Nabla(arr) => &mut arr[20],
        }
    }

    pub fn cloud_grade_mut(&mut self) -> &mut u32 {
        match self {
            Score::ExceedGear(arr) => &mut arr[19],
            Score::Nabla(arr) => &mut arr[21],
        }
    }

    pub fn cloud_ex_score_mut(&mut self) -> Option<&mut u32> {
        match self {
            Score::ExceedGear(_) => None,
            Score::Nabla(arr) => Some(&mut arr[19]),
        }
    }

    pub fn to_property(self) -> Vec<u32> {
        match self {
            Score::ExceedGear(arr) => arr.to_vec(),
            Score::Nabla(arr) => arr.to_vec(),
        }
    }
}
