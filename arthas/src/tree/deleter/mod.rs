
use super::Tree;


pub struct Deleter {}

impl Deleter {
    pub fn delete(tree: &mut Tree, id: &str) {
        thread_trace!("delete id: {}", id);
        let rc_item = tree.id_map.remove(id);
        if rc_item.is_none() {
            return;
        }

        let rc_item = rc_item.unwrap();

        for (field_int, rc_node) in &rc_item.read().unwrap().nodes {
            let (clear, deleted_rc_node, root) = rc_node.write().unwrap().delete(id);
            if clear {
                let rc_node = tree.root.remove(field_int);
                if rc_node.is_some() {
                    rc_node.unwrap().destroy();
                }
            } else if deleted_rc_node.is_some() {
                deleted_rc_node.unwrap().destroy();
            }

            if root.is_some() {
                tree.root.insert(field_int.clone(), root.unwrap());
            }
        }

        rc_item.destroy();
    }

    pub fn clear(tree: &mut Tree) {
        tree.id_map.clear();
        tree.root.clear();
    }
}
