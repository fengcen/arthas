
use item::Id;
use serde_json::Value;
use super::rc::{RcItem, RcChild, RcNode};
use super::Tree;

pub struct Inserter {}

impl Inserter {
    pub fn insert(tree: &mut Tree, id: Id, value: Value) {
        thread_trace!("insert id: {}", id);
        let rc_item = RcItem::new(id.clone(), value);
        tree.id_map.insert(id.clone(), rc_item.clone());
        let datas = rc_item.read().unwrap().datas.clone();
        let mut is_min = true;
        let mut is_max = true;

        for (field_int, rc_data) in datas {
            if rc_data.read().unwrap().can_index() {
                let rc_child = RcChild::new(rc_data, rc_item.clone());
                let root_exists = tree.root.contains_key(&field_int);

                if root_exists {
                    let node_option = tree.root
                        .get(&field_int)
                        .unwrap()
                        .write()
                        .unwrap()
                        .insert(field_int.clone(),
                                id.clone(),
                                rc_child,
                                &mut is_min,
                                &mut is_max);

                    if node_option.is_some() {
                        let rc_node = node_option.unwrap();
                        if is_min {
                            thread_trace!("found min node: {:?}",
                                          rc_node.read().unwrap().get_value());
                            tree.min.insert(field_int, rc_node);
                        } else if is_max {
                            thread_trace!("found max node: {:?}",
                                          rc_node.read().unwrap().get_value());
                            tree.max.insert(field_int, rc_node);
                        } else {
                            unreachable!()
                        }
                    }
                } else {
                    let rc_node = RcNode::new(id.clone(), rc_child);
                    rc_node.write().unwrap().self_rc = Some(rc_node.clone());
                    rc_item.write().unwrap().nodes.insert(field_int.clone(), rc_node.clone());
                    tree.root.insert(field_int, rc_node);
                }
            }
        }
    }
}
