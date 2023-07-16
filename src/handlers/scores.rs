use crate::types::game::GameScores;
use crate::types::tachi::{Difficulty, HitMeta, Import, ImportScore, Judgements, TachiLamp};
use crate::{helpers, CONFIGURATION, TACHI_IMPORT_URL};
use anyhow::Result;
use either::Either;
use log::info;

pub fn process_scores(scores: GameScores) -> Result<()> {
    let card = scores.ref_id;
    if !CONFIGURATION.cards.whitelist.is_empty() && !CONFIGURATION.cards.whitelist.contains(&card) {
        info!("Card {} is not whitelisted, skipping score(s)", card);
        return Ok(());
    }

    let tracks = match scores.tracks {
        Either::Left(track) => vec![track],
        Either::Right(tracks) => tracks,
    };

    let time_achieved = std::time::UNIX_EPOCH
        .elapsed()
        .map(|duration| duration.as_millis())
        .map_err(|err| anyhow::anyhow!("Could not get time from System {:#}", err))?;

    let scores = tracks
        .into_iter()
        .map(|track| ImportScore {
            score: track.score,
            lamp: TachiLamp::from(track.clear_type),
            match_type: "sdvxInGameID".to_string(),
            identifier: track.music_id.to_string(),
            difficulty: Difficulty::from(track.music_type),
            time_achieved,
            judgements: Judgements {
                critical: track.critical,
                near: track.near,
                miss: track.error,
            },
            hit_meta: HitMeta {
                fast: track.judge[0],
                slow: track.judge[6],
                max_combo: track.max_chain,
                ex_score: if track.ex_score == 0 && track.score != 0 {
                    None
                } else {
                    Some(track.ex_score)
                },
                gauge: track.effective_rate as f32 / 100.0,
            },
        })
        .collect();

    let import = Import {
        meta: Default::default(),
        classes: None,
        scores,
    };

    helpers::call_tachi("POST", TACHI_IMPORT_URL.as_str(), Some(import))?;
    info!("Successfully imported score(s) for card {}", card);

    Ok(())
}
