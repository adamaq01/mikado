use crate::mikado::{send_import, CONFIGURATION};
use crate::types::game::GameSave;
use crate::types::tachi::{Import, ImportClasses, SkillLevel};
use anyhow::Result;
use log::info;

pub fn process_course(course: GameSave) -> Result<()> {
    let card = course.ref_id;
    if !CONFIGURATION.cards.whitelist.is_empty() && !CONFIGURATION.cards.whitelist.contains(&card) {
        info!("Card {} is not whitelisted, skipping class update", card);
        return Ok(());
    }

    let import = Import {
        meta: Default::default(),
        classes: Some(ImportClasses {
            dan: SkillLevel::from(course.skill_level),
        }),
        scores: vec![],
    };

    send_import(import)?;
    info!("Successfully updated class for card {}", card);

    Ok(())
}
