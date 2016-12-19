
use serde_json::Value;
use super::super::rc::RcNode;
use super::super::math::Math;
use super::super::Tree;


pub enum EntranceType {
    Root,
    Min,
    Max,
    None,
}

impl EntranceType {
    pub fn new(tree: &Tree, field_int: &str, value: &Value) -> EntranceType {
        let root = tree.root.get(field_int);
        let min = tree.min.get(field_int);
        let max = tree.max.get(field_int);

        if root.is_none() {
            EntranceType::None
        } else if min.is_some() && max.is_none() {
            let root_rate = get_rate(root.unwrap(), value);
            let min_rate = get_rate(min.unwrap(), value);
            if root_rate > min_rate {
                EntranceType::Root
            } else {
                EntranceType::Min
            }
        } else if min.is_none() && max.is_some() {
            let max_rate = get_rate(max.unwrap(), value);
            let root_rate = get_rate(root.unwrap(), value);
            if root_rate > max_rate {
                EntranceType::Root
            } else {
                EntranceType::Max
            }
        } else if min.is_some() && max.is_some() {
            let root_rate = get_rate(root.unwrap(), value);
            let min_rate = get_rate(min.unwrap(), value);
            let max_rate = get_rate(max.unwrap(), value);
            let maximum = vec![root_rate, min_rate, max_rate]
                .into_iter()
                .fold(0.0, |a, b| if a > b { a } else { b });

            if maximum == min_rate {
                EntranceType::Min
            } else if maximum == max_rate {
                EntranceType::Max
            } else {
                EntranceType::Root
            }
        } else {
            EntranceType::Root
        }
    }
}

fn get_rate(rc_node: &RcNode, value: &Value) -> f64 {
    rc_node.read().unwrap().get_value().get_rate(value)
}
