use crate::types::game::GameSave;
use crate::types::tachi::{Import, ImportClasses, SkillLevel};
use crate::{helpers, CONFIGURATION, TACHI_IMPORT_URL};
use anyhow::Result;
use log::info;

pub fn process_save(save: GameSave) -> Result<()> {
    if save.ref_id.is_none() {
        info!("Guest play, skipping class update");
        return Ok(());
    }
    let card = if let Some(card) = helpers::get_current_card_id() {
        if !CONFIGURATION.cards.whitelist.is_empty()
            && !CONFIGURATION.cards.whitelist.contains(&card)
        {
            info!("Card {} is not whitelisted, skipping class update", card);
            return Ok(());
        }

        card
    } else {
        info!("Card ID is not set, skipping class update");
        return Ok(());
    };

    let import = Import {
        meta: Default::default(),
        classes: Some(ImportClasses {
            dan: SkillLevel::from(save.skill_level),
        }),
        scores: vec![],
    };

    helpers::call_tachi("POST", TACHI_IMPORT_URL.as_str(), Some(import))?;
    info!("Successfully updated class for card {}", card);

    Ok(())
}
