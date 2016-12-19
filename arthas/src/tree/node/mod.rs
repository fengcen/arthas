
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::fmt;
use serde_json::Value;
use item::{Id, FieldInt};
use super::comparision::Comparision;
use super::action::SearchAction;
use super::rc::RcChild;
use super::math::Math;
use super::rc::DataType;
use super::rc::RcNode;
use super::searcher::task::Sub;


pub type Group = Arc<RwLock<HashMap<Id, RcChild>>>;
pub type Groups = Vec<Group>;


#[derive(Clone)]
pub struct Node {
    pub _type: DataType,
    pub self_rc: Option<RcNode>,
    pub group: Group,
    pub parent: Option<RcNode>,
    pub left: Option<RcNode>,
    pub right: Option<RcNode>,
}

impl Node {
    pub fn new(id: Id, rc_child: RcChild) -> Node {
        let _type = rc_child.read().unwrap().get_type();

        let mut map = HashMap::new();
        map.insert(id, rc_child);

        Node {
            _type: _type,
            self_rc: None,
            group: Arc::new(RwLock::new(map)),
            parent: None,
            left: None,
            right: None,
        }
    }

    pub fn insert(&mut self,
                  field_int: FieldInt,
                  id: Id,
                  rc_child: RcChild,
                  is_min: &mut bool,
                  is_max: &mut bool)
                  -> Option<RcNode> {
        match self._type {
            DataType::Number | DataType::String => {
                if Math::gt(&rc_child.read().unwrap().get_value(), &self.get_value()) {
                    if self.right.is_some() {
                        self.insert_continue_right(field_int, id, rc_child, is_min, is_max)
                    } else {
                        self.set_right(field_int, id, rc_child, is_max)
                    }
                } else if Math::lt(&rc_child.read().unwrap().get_value(), &self.get_value()) {
                    if self.left.is_some() {
                        self.insert_continue_left(field_int, id, rc_child, is_min, is_max)
                    } else {
                        self.set_left(field_int, id, rc_child, is_min)
                    }
                } else {
                    self.insert_to_current(field_int, id, rc_child);
                    *is_min = false;
                    *is_max = false;
                    None
                }
            }
            _ => unreachable!(),
        }
    }

    fn insert_to_current(&mut self, field_int: FieldInt, id: Id, rc_child: RcChild) {
        {
            let child = rc_child.read().unwrap();
            child.item
                .write()
                .unwrap()
                .nodes
                .insert(field_int, self.self_rc.clone().unwrap());
        }

        self.group.write().unwrap().insert(id, rc_child);
    }

    fn insert_continue_right(&mut self,
                             field_int: FieldInt,
                             id: Id,
                             rc_child: RcChild,
                             is_min: &mut bool,
                             is_max: &mut bool)
                             -> Option<RcNode> {
        if *is_min {
            *is_min = false;
        }

        self.right
            .as_mut()
            .unwrap()
            .write()
            .unwrap()
            .insert(field_int, id, rc_child, is_min, is_max)
    }

    fn insert_continue_left(&mut self,
                            field_int: FieldInt,
                            id: Id,
                            rc_child: RcChild,
                            is_min: &mut bool,
                            is_max: &mut bool)
                            -> Option<RcNode> {
        if *is_max {
            *is_max = false;
        }

        self.left
            .as_mut()
            .unwrap()
            .write()
            .unwrap()
            .insert(field_int, id, rc_child, is_min, is_max)
    }

    fn set_left(&mut self,
                field_int: FieldInt,
                id: Id,
                rc_child: RcChild,
                is_min: &mut bool)
                -> Option<RcNode> {
        let rc_node = RcNode::new(id, rc_child.clone());
        let child = rc_child.read()
            .unwrap();
        child.item
            .write()
            .unwrap()
            .nodes
            .insert(field_int, rc_node.clone());

        {
            let mut node = rc_node.write().unwrap();
            node.parent = self.self_rc.clone();
            node.self_rc = Some(rc_node.clone());
        }

        self.left = Some(rc_node.clone());

        if *is_min { Some(rc_node) } else { None }
    }

    fn set_right(&mut self,
                 field_int: FieldInt,
                 id: Id,
                 rc_child: RcChild,
                 is_max: &mut bool)
                 -> Option<RcNode> {
        let rc_node = RcNode::new(id, rc_child.clone());
        let child = rc_child.read().unwrap();
        child.item
            .write()
            .unwrap()
            .nodes
            .insert(field_int, rc_node.clone());

        {
            let mut node = rc_node.write().unwrap();
            node.parent = self.self_rc.clone();
            node.self_rc = Some(rc_node.clone());
        }

        self.right = Some(rc_node.clone());

        if *is_max { Some(rc_node) } else { None }
    }

    pub fn search_root(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        let action_option = self.search_current(stopped, groups, sub);
        if action_option.is_some() {
            self.go_right_or_go_left(action_option.unwrap(), stopped, groups, sub);
        }
    }

    fn search_max_root(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        let action_option = self.search_min_or_max_current(stopped, groups, sub);
        if action_option.is_some() {
            self.search_min_or_max_go_right_or_go_left(action_option.unwrap(),
                                                       stopped,
                                                       groups,
                                                       sub);
        }
    }

    fn search_min_root(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        let action_option = self.search_min_or_max_current(stopped, groups, sub);
        if action_option.is_some() {
            self.search_min_or_max_go_right_or_go_left(action_option.unwrap(),
                                                       stopped,
                                                       groups,
                                                       sub);
        }
    }

    pub fn search_max(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        let action_option = self.search_current(stopped, groups, sub);
        if action_option.is_some() {
            let action = action_option.unwrap();

            if action.take {
                self.append_to(groups);
            }

            let mut should_stop = true;

            if action.fold_left {
                self.fold_left_top(groups);
            } else if action.go_left {
                if self.left.is_some() {
                    self.go_left(stopped, groups, sub);
                }

                if self.parent.is_some() {
                    should_stop = false;
                    self.go_top_max(stopped, groups, sub);
                }
            }

            if should_stop {
                return stop(stopped, sub);
            }
        }
    }

    pub fn search_min(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        let action_option = self.search_current(stopped, groups, sub);
        if action_option.is_some() {
            let action = action_option.unwrap();

            if action.take {
                self.append_to(groups);
            }

            let mut should_stop = true;

            if action.fold_right {
                self.fold_right_top(groups);
            } else if action.go_right {
                if self.right.is_some() {
                    self.go_right(stopped, groups, sub);
                }

                if self.parent.is_some() {
                    should_stop = false;
                    self.go_top_min(stopped, groups, sub);
                }
            }

            if should_stop {
                return stop(stopped, sub);
            }
        }
    }

    fn go_right(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        thread_trace!("search min, go right");
        self.right.as_ref().unwrap().read().unwrap().search_min_root(stopped, groups, sub);
    }

    fn go_left(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        thread_trace!("search max, go left");
        self.left.as_ref().unwrap().read().unwrap().search_max_root(stopped, groups, sub);
    }

    fn go_top_max(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        thread_trace!("search max, go top");
        self.parent.as_ref().unwrap().read().unwrap().search_max(stopped, groups, sub);
    }

    fn go_top_min(&self, stopped: &AtomicBool, groups: &mut Groups, sub: &mut Sub) {
        thread_trace!("search min, go top");
        self.parent.as_ref().unwrap().read().unwrap().search_min(stopped, groups, sub);
    }

    fn search_min_or_max_current(&self,
                                 stopped: &AtomicBool,
                                 groups: &mut Groups,
                                 sub: &mut Sub)
                                 -> Option<SearchAction> {
        if stopped.load(Ordering::SeqCst) {
            thread_trace!("search min current, other threads had stopped, return none");
            return None;
        }

        let action_option = self.compare_self(&sub.comparisions);
        if action_option.is_none() {
            thread_trace!("search min current, action is none, return none");
            return None;
        }

        let action = action_option.unwrap();
        self.fold_action(&action, groups);

        if action.is_stopped() {
            thread_trace!("search min current, action is stopped, return none");
            return None;
        }

        thread_trace!("search min current, found some action: {:?}", action);
        Some(action)
    }

    fn search_current(&self,
                      stopped: &AtomicBool,
                      groups: &mut Groups,
                      sub: &mut Sub)
                      -> Option<SearchAction> {
        if stopped.load(Ordering::SeqCst) {
            thread_trace!("other threads had stopped, stopping current.");
            return None;
        }

        let action_option = self.compare_self(&sub.comparisions);
        if action_option.is_none() {
            stop(stopped, sub);
            thread_trace!("action is none, stopping sub.");
            return None;
        }

        let action = action_option.unwrap();
        self.fold_action(&action, groups);

        if action.is_stopped() {
            stop(stopped, sub);
            thread_trace!("action is stopped, stopping sub.");
            return None;
        }

        thread_trace!("found some action: {:?}", action);
        Some(action)
    }

    fn search_min_or_max_go_right_or_go_left(&self,
                                             action: SearchAction,
                                             stopped: &AtomicBool,
                                             groups: &mut Groups,
                                             sub: &mut Sub) {
        if action.go_right && self.right.is_some() {
            self.right
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .search_min_root(stopped, groups, sub);
        }

        if action.go_left && self.left.is_some() {
            self.left
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .search_min_root(stopped, groups, sub);

        }
    }

    fn go_right_or_go_left(&self,
                           action: SearchAction,
                           stopped: &AtomicBool,
                           groups: &mut Groups,
                           sub: &mut Sub) {
        let mut no_right_go = true;
        let mut no_left_go = true;

        if action.go_right && self.right.is_some() {
            no_right_go = false;
            self.right
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .search_root(stopped, groups, sub);
        }

        if action.go_left && self.left.is_some() {
            no_left_go = false;
            self.left
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .search_root(stopped, groups, sub);

        }

        if no_left_go && no_right_go {
            return stop(stopped, sub);
        }
    }

    fn fold_action(&self, action: &SearchAction, groups: &mut Groups) {
        if action.take {
            thread_trace!("fold action, take current childs: {}",
                          self.group.read().unwrap().len());

            self.append_to(groups);
        }

        if action.fold_right {
            thread_trace!("fold right");
            self.fold_right_to(groups);
        }

        if action.fold_left {
            thread_trace!("fold left");
            self.fold_left_to(groups);
        }
    }

    fn fold_left_top(&self, groups: &mut Groups) {
        self.fold_left_to(groups);

        if self.parent.is_some() {
            self.parent.as_ref().unwrap().read().unwrap().fold_current_left(groups);
        }
    }

    fn fold_right_top(&self, groups: &mut Groups) {
        self.fold_right_to(groups);

        if self.parent.is_some() {
            self.parent.as_ref().unwrap().read().unwrap().fold_current_right(groups);
        }
    }

    fn fold_current_left(&self, groups: &mut Groups) {
        self.append_to(groups);
        self.fold_left_to(groups);
    }

    fn fold_current_right(&self, groups: &mut Groups) {
        self.append_to(groups);
        self.fold_right_to(groups);
    }

    fn fold_left_to(&self, groups: &mut Groups) {
        if self.left.is_some() {
            self.left.as_ref().unwrap().read().unwrap().fold_to(groups);
        }
    }

    fn fold_right_to(&self, groups: &mut Groups) {
        if self.right.is_some() {
            self.right.as_ref().unwrap().read().unwrap().fold_to(groups);
        }
    }

    fn fold_to(&self, groups: &mut Groups) {
        self.append_to(groups);
        self.fold_right_to(groups);
        self.fold_left_to(groups);
    }

    fn append_to(&self, groups: &mut Groups) {
        groups.push(self.group.clone());
    }

    fn compare_self(&self, comparisions: &[Comparision]) -> Option<SearchAction> {
        let mut prev_action = None;

        for comparision in comparisions {
            let compared_action = comparision.compare(&self.get_value());
            if compared_action.is_none() {
                return None;
            }

            if prev_action.is_none() {
                prev_action = compared_action;
            } else {
                let merged_action = prev_action.as_ref().unwrap().merge(&compared_action.unwrap());
                if merged_action.is_none() {
                    return None;
                } else {
                    prev_action = merged_action;
                }
            }
        }

        prev_action
    }

    pub fn delete(&mut self, id: &str) -> (bool, Option<RcNode>, Option<RcNode>) {
        if self.group.read().unwrap().len() == 1 {
            let (clear, root) = self.delete_self();
            (clear, self.self_rc.take(), root)
        } else {
            let rc_child = self.group.write().unwrap().remove(id);
            if rc_child.is_some() {
                rc_child.unwrap().destroy();
            }

            (false, None, None)
        }
    }

    fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    fn delete_self(&mut self) -> (bool, Option<RcNode>) {
        let mut clear = false;
        let mut root = None;

        if self.has_no_child_node() {
            if self.is_root() {
                clear = true;
            } else {
                self.parent.as_ref().unwrap().write().unwrap().delete_child_node(&self.self_rc);
            }
        } else if self.has_two_child_node() {
            let is_root = self.is_root();
            let need_link_rc_node = self.take_right_min_node();

            {
                let mut need_link_node = need_link_rc_node.write().unwrap();

                self.replace_parent_child_node(need_link_rc_node.clone());

                if is_root {
                    need_link_node.parent = None;
                } else {
                    need_link_node.parent = Some(self.parent.take().unwrap());
                }

                let left_rc_node = self.left.take().unwrap();
                left_rc_node.write().unwrap().parent = Some(need_link_rc_node.clone());
                need_link_node.left = Some(left_rc_node);

                if self.right.is_some() {
                    let right_rc_node = self.right.take().unwrap();
                    right_rc_node.write().unwrap().parent = Some(need_link_rc_node.clone());
                    need_link_node.right = Some(right_rc_node);
                }
            }

            if is_root {
                clear = true;
                root = Some(need_link_rc_node);
            }
        } else if self.left.is_some() {
            let rc_node = self.left.take().unwrap();

            if self.is_root() {
                rc_node.write().unwrap().parent = None;
                clear = true;
                root = Some(rc_node);
            } else {
                rc_node.write().unwrap().parent = self.parent.clone();
                self.replace_parent_child_node(rc_node);
            }
        } else if self.right.is_some() {
            let rc_node = self.right.take().unwrap();

            if self.is_root() {
                rc_node.write().unwrap().parent = None;
                clear = true;
                root = Some(rc_node);
            } else {
                rc_node.write().unwrap().parent = self.parent.clone();
                self.replace_parent_child_node(rc_node);
            }
        } else {
            unreachable!()
        }

        (clear, root)
    }

    fn right_child_no_left(&self) -> bool {
        self.right.as_ref().unwrap().read().unwrap().left.is_none()
    }

    fn left_child_no_left(&self) -> bool {
        self.left.as_ref().unwrap().read().unwrap().left.is_none()
    }

    fn take_left_min_node(&mut self) -> RcNode {
        if self.left_child_no_left() {
            self.left.take().unwrap()
        } else {
            self.left.as_ref().unwrap().write().unwrap().take_left_min_node()
        }
    }

    fn take_right_min_node(&mut self) -> RcNode {
        if self.right_child_no_left() {
            self.right.take().unwrap()
        } else {
            self.right.as_ref().unwrap().write().unwrap().take_left_min_node()
        }
    }

    fn replace_parent_child_node(&self, rc_node: RcNode) {
        let mut parent = self.parent.as_ref().unwrap().write().unwrap();
        parent.replace_child_node(&self.self_rc, rc_node);
    }

    fn replace_child_node(&mut self, from: &Option<RcNode>, to: RcNode) {
        if self.left == *from {
            self.left = Some(to);
        } else {
            self.right = Some(to);
        }
    }

    fn delete_child_node(&mut self, self_rc: &Option<RcNode>) {
        if self.left == *self_rc {
            self.left = None;
        } else {
            self.right = None;
        }
    }

    fn has_no_child_node(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    fn has_two_child_node(&self) -> bool {
        self.left.is_some() && self.right.is_some()
    }

    pub fn get_value(&self) -> Value {
        for child in self.group.read().unwrap().values() {
            return child.read().unwrap().get_value();
        }

        unreachable!()
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.group)
    }
}

#[inline]
fn stop(stopped: &AtomicBool, sub: &mut Sub) {
    sub.stopped = true;
    stopped.store(true, Ordering::SeqCst);
}
