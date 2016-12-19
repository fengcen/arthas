
use super::cmp::Cmp;
use serde_json::Value;
use super::math::Math;
use item::FieldInt;
use super::action::SearchAction;
use super::searcher::meet::Meet;


#[derive(Debug, Clone)]
pub struct Comparision {
    pub field_int: FieldInt,
    pub cmp: Cmp,
    pub other: Value,
}

impl Comparision {
    pub fn new(field_int: FieldInt, cmp: Cmp, other: Value) -> Comparision {
        Comparision {
            field_int: field_int,
            cmp: cmp,
            other: other,
        }
    }

    pub fn _match(&self, value: &Value) -> bool {
        value.meet(&self.cmp, &self.other)
    }

    pub fn compare(&self, current: &Value) -> Option<SearchAction> {
        match self.cmp {
            Cmp::Eq => {
                if Math::eq(current, &self.other) {
                    Some(SearchAction::new().take())
                } else if Math::lt(current, &self.other) {
                    Some(SearchAction::new().go_right())
                } else {
                    Some(SearchAction::new().go_left())
                }
            }
            Cmp::Ne => {
                if Math::eq(current, &self.other) {
                    Some(SearchAction::new().fold_left().fold_right())
                } else if Math::lt(current, &self.other) {
                    Some(SearchAction::new().take().fold_left().go_right())
                } else {
                    Some(SearchAction::new().take().fold_right().go_left())
                }
            }
            Cmp::Gt => {
                if Math::gt(current, &self.other) {
                    Some(SearchAction::new().take().fold_right().go_left())
                } else if Math::lt(current, &self.other) {
                    Some(SearchAction::new().go_right())
                } else {
                    Some(SearchAction::new().fold_right())
                }
            }
            Cmp::Lt => {
                if Math::lt(current, &self.other) {
                    Some(SearchAction::new().take().fold_left().go_right())
                } else if Math::gt(current, &self.other) {
                    Some(SearchAction::new().go_left())
                } else {
                    Some(SearchAction::new().fold_left())
                }
            }
            Cmp::Ge => {
                if Math::eq(current, &self.other) || Math::gt(current, &self.other) {
                    Some(SearchAction::new().take().fold_right())
                } else {
                    Some(SearchAction::new().go_right())
                }
            }
            Cmp::Le => {
                if Math::eq(current, &self.other) || Math::lt(current, &self.other) {
                    Some(SearchAction::new().take().fold_left())
                } else {
                    Some(SearchAction::new().go_left())
                }
            }
        }
    }
}
