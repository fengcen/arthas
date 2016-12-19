
use std::collections::HashMap;
use item::Id;
use query::Query;

pub trait Schema: Sized {
    fn session<'a>() -> Query<'a, Self>;
    fn get_field_type_map() -> FieldTypeMap;
    fn get_field_int_map() -> FieldIntMap;
    fn is_one() -> bool;
    fn new_empty() -> Self;
    fn new_deep_empty() -> Self;
    fn has_id() -> bool;
    fn set_id(&mut self, id: Id);
    fn get_id(&self) -> Id;
}

pub trait SchemaRename {
    fn get_rename_map() -> RenameMap;
}

pub type FieldTypeMap = HashMap<String, FieldType>;
pub type FieldIntMap = HashMap<String, String>;
pub type RenameMap = HashMap<String, String>;

#[derive(Debug, Clone)]
pub enum FieldType {
    Atomic(String),
    Array(FieldTypeMap),
    Object(FieldTypeMap),
    Struct(FieldTypeMap),
}

use serde::{Serialize, Deserialize};
use std::fmt::Debug;

pub trait Structure
    : 'static + Serialize + Deserialize + Default + Clone + Debug + PartialEq + Send + Sync + Schema + SchemaRename
    {
}
impl<T: 'static + Serialize + Deserialize + Default + Clone + Debug + PartialEq + Send + Sync + Schema + SchemaRename> Structure for T {}

pub fn get_unique_int_str(field: &str) -> String {
    let mut integer = 0;

    for (index, ch) in field.chars().enumerate() {
        integer += index * (ch as usize);
    }

    integer.to_string()
}
