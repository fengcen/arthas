
use std::collections::HashMap;
use item::Id;
use query::Query;

pub trait Arthas: Sized {
    fn get_struct_name() -> String;
    fn session<'a>() -> Query<'a, Self>;
    fn get_field_type_map() -> FieldTypeMap;
    fn get_field_int_map() -> FieldIntMap;
    fn is_one() -> bool;
    fn new_empty() -> Self;
    fn new_deep_empty() -> Self;
    fn has_id() -> bool;
    fn set_id(&mut self, id: Id);
    fn get_id(&self) -> Id;
    fn get_rename_map() -> HashMap<String, String>;
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
    : 'static + Serialize + Deserialize + Default + Clone + Debug + PartialEq + Send + Sync + Arthas
    {
}
impl<T: 'static + Serialize + Deserialize + Default + Clone + Debug + PartialEq + Send + Sync + Arthas> Structure for T {}

pub fn get_unique_int_str(field: &str) -> String {
    let mut integer = 0;

    for (index, ch) in field.chars().enumerate() {
        integer += index * (ch as usize);
    }

    integer.to_string()
}
