
use serde_json::Value;


#[derive(Clone, Debug)]
pub enum DataType {
    String,
    Number,
    Bool,
    Array,
    Object,
    Null,
}

impl<'a> From<&'a Value> for DataType {
    fn from(value: &Value) -> DataType {
        match *value {
            Value::String(_) => DataType::String,
            Value::Number(_) => DataType::Number,
            Value::Bool(_) => DataType::Bool,
            Value::Null => DataType::Null,
            Value::Array(_) => DataType::Array,
            Value::Object(_) => DataType::Object,
        }
    }
}
