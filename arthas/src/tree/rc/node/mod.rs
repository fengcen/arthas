
use std::sync::RwLock;
use std::ops::Deref;
use std::ops::DerefMut;
use item::Id;
use super::RcChild;
use super::super::node::Node;


#[derive(Clone, Debug)]
pub struct RcNode(*mut RwLock<Node>);

impl RcNode {
    pub fn new(id: Id, rc_child: RcChild) -> RcNode {
        RcNode(Box::into_raw(Box::new(RwLock::new(Node::new(id, rc_child)))))
    }

    pub fn destroy(self) {
        let rc_node = unsafe { Box::from_raw(self.0) };
        let node = rc_node.write().unwrap();
        let mut group = node.group.write().unwrap();

        for (_, rc_child) in group.drain() {
            rc_child.destroy();
        }
    }
}

unsafe impl Send for RcNode {}
unsafe impl Sync for RcNode {}

impl Deref for RcNode {
    type Target = RwLock<Node>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for RcNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl PartialEq for RcNode {
    fn eq(&self, other: &RcNode) -> bool {
        self.0 == other.0
    }
}
