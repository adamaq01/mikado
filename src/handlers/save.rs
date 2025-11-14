use crate::types::game::GameSave;
use crate::types::tachi::{Import, ImportClasses, SkillLevel};
use crate::{helpers, TACHI_IMPORT_URL};
use anyhow::Result;
use log::info;

pub fn process_save(save: GameSave) -> Result<()> {
    if save.ref_id.is_none() {
        info!("Guest play, skipping class update");
        return Ok(());
    }

    let Some(user) = helpers::get_current_user() else {
        info!("User is not set, skipping class update");
        return Ok(());
    };

    let import = Import {
        meta: Default::default(),
        classes: Some(ImportClasses {
            dan: SkillLevel::from(save.skill_level),
        }),
        scores: vec![],
    };

    helpers::call_tachi("POST", TACHI_IMPORT_URL.as_str(), user.card_config.api_key, Some(import))?;
    info!("Successfully updated class for card {0}", user.card_id);

    Ok(())
}
