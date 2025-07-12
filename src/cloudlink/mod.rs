mod ext;

use crate::types::cloudlink::{Chart, Score};
use crate::types::tachi::{TachiDifficulty, TachiLamp};
use crate::{helpers, mikado, TACHI_PBS_URL};
use anyhow::Result;
use dynfmt::Format;
use ext::HashMapExt;
use kbinxml::{Node, Value, ValueArray};
use log::info;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

fn build_response_base(scores: Vec<Node>) -> Node {
    Node::with_nodes(
        "response",
        vec![Node::with_nodes(
            "game",
            vec![Node::with_nodes("music", scores)],
        )],
    )
}

// TODO: Refactor this whole mess
pub fn process_pbs(user: &str, music: &Node) -> Result<Node> {
    let url = dynfmt::SimpleCurlyFormat.format(TACHI_PBS_URL.as_str(), [user])?;
    let response: serde_json::Value = helpers::request_tachi("GET", url, None::<()>)?;
    let body = response["body"].as_object().ok_or(anyhow::anyhow!(
        "Could not parse response body from Tachi PBs API"
    ))?;
    let pbs = body["pbs"]
        .as_array()
        .ok_or(anyhow::anyhow!("Could not parse PBs from Tachi PBs API"))?;
    let charts = body["charts"]
        .as_array()
        .ok_or(anyhow::anyhow!("Could not parse charts from Tachi PBs API"))?;
    let charts = charts
        .iter()
        .map(|chart| {
            let chart_id = chart["chartID"].as_str().ok_or(anyhow::anyhow!(
                "Could not parse chart ID from Tachi PBs API"
            ))?;
            let song_id = chart["data"]["inGameID"].as_u64().ok_or(anyhow::anyhow!(
                "Could not parse ingame ID from Tachi PBs API"
            ))? as u32;
            let difficulty =
                match serde_json::from_value::<TachiDifficulty>(chart["difficulty"].clone()) {
                    Ok(difficulty) => u32::from(difficulty) as u8,
                    Err(_) => 3,
                };
            Ok((
                chart_id,
                Chart {
                    song_id,
                    difficulty,
                },
            ))
        })
        .collect::<Result<HashMap<&str, Chart>>>()?;

    let mut scores = HashMap::with_capacity(music.children().len() + pbs.len());
    for pb in music.children() {
        let score = pb
            .children()
            .first()
            .ok_or(anyhow::anyhow!("Could not find param node"))?;
        if let Value::Array(ValueArray::U32(value)) = score
            .value()
            .ok_or(anyhow::anyhow!("Could not find value in param node"))?
        {
            let song_id = value[0];
            let difficulty = value[1] as u8;
            let chart = Chart {
                song_id,
                difficulty,
            };
            let score = Score::from_slice(value)?;
            scores.insert(chart, score);
        }
    }

    for pb in pbs {
        let chart_id = pb["chartID"].as_str().ok_or(anyhow::anyhow!(
            "Could not parse chart ID from Tachi PBs API"
        ))?;
        let chart = charts
            .get(chart_id)
            .ok_or(anyhow::anyhow!("Could not find chart"))?;
        let score = pb["scoreData"]["score"].as_u64().ok_or(anyhow::anyhow!(
            "Could not parse PB score from Tachi PBs API"
        ))?;
        let lamp: u32 = match serde_json::from_value::<TachiLamp>(pb["scoreData"]["lamp"].clone()) {
            Ok(lamp) => match lamp {
                TachiLamp::MaxxiveClear
                    if !mikado::GAME_PROPERTIES
                        .get()
                        .map(|p| p.has_maxxive_support())
                        .unwrap_or_default() =>
                {
                    TachiLamp::ExcessiveClear
                }
                _ => lamp,
            }
            .into(),
            Err(_) => 0,
        };
        let grade = pb["scoreData"]["enumIndexes"]["grade"]
            .as_u64()
            .ok_or(anyhow::anyhow!(
                "Could not parse PB grade from Tachi PBs API"
            ))?
            + 1;
        let grade = if grade >= 11 { 10 } else { grade };

        let entry = scores.entry(*chart);
        match entry {
            Entry::Occupied(mut entry) => {
                let base_score = entry.get_mut();
                *base_score.cloud_score_mut() = score as u32;
                *base_score.cloud_clear_mut() = lamp as u32;
                *base_score.cloud_grade_mut() = grade as u32;
            }
            Entry::Vacant(entry) => {
                let score = Score::from_cloud(score as u32, lamp as u8, grade as u8);
                entry.insert(score);
            }
        }
    }

    let response = build_response_base(scores.to_properties());
    info!("Successfully injected Tachi PBs as Cloud scores");

    Ok(response)
}
