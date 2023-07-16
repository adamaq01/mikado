use crate::types::cloudlink::{Chart, Score};
use kbinxml::{Node, Value, ValueArray};
use std::collections::HashMap;

pub(crate) trait HashMapExt {
    fn to_properties(self) -> Vec<Node>;
}

impl HashMapExt for HashMap<Chart, Score> {
    fn to_properties(self) -> Vec<Node> {
        self.into_iter()
            .map(|(chart, score)| {
                let mut property = score.to_property();
                property[0] = chart.song_id;
                property[1] = chart.difficulty as u32;

                Node::with_nodes(
                    "info",
                    vec![Node::with_value(
                        "param",
                        Value::Array(ValueArray::U32(property)),
                    )],
                )
            })
            .collect()
    }
}
