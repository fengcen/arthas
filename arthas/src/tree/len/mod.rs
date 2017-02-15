
use serde_json::{self, Value};
use super::rc::RcData;


pub trait Len {
    fn can_len(&self) -> bool;
    fn create_len_rc_data(&self) -> Option<RcData>;
}

impl Len for Value {
    fn can_len(&self) -> bool {
        match *self {
            Value::Array(_) | Value::Object(_) | Value::String(_) => true,
            _ => false,
        }
    }

    fn create_len_rc_data(&self) -> Option<RcData> {
        match *self {
            Value::Array(ref array) => create_rc_data_by_usize(array.len()),
            Value::Object(ref map) => create_rc_data_by_usize(map.len()),
            Value::String(ref string) => create_rc_data_by_usize(string.len()),
            _ => None,
        }
    }
}

#[inline]
fn create_rc_data_by_usize(len: usize) -> Option<RcData> {
    Some(RcData::new(Box::into_raw(Box::new(Value::Number(serde_json::Number::from(len))))))
}
