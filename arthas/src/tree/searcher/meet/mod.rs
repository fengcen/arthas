
use super::super::cmp::Cmp;
use super::super::math::Math;
use serde_json::Value;


pub trait Meet {
    fn meet(&self, cmp: &Cmp, other: &Value) -> bool;
}

impl Meet for Value {
    fn meet(&self, cmp: &Cmp, other: &Value) -> bool {
        match *cmp {
            Cmp::Eq => Math::eq(self, other),
            Cmp::Gt => Math::gt(self, other),
            Cmp::Lt => Math::lt(self, other),
            Cmp::Ge => Math::ge(self, other),
            Cmp::Le => Math::le(self, other),
            Cmp::Ne => Math::ne(self, other),
        }
    }
}
