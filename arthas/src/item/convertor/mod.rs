
use super::ItemWrapper;
use serde_json::{self, Value};
use traits::{Structure, FieldIntMap};
use error::Error;
use encoder;


pub trait Convertor {
    fn to_wrapper<T: Structure>(&self,
                                field_int_map: &FieldIntMap)
                                -> Result<ItemWrapper<T>, Error>;
}

impl Convertor for Value {
    fn to_wrapper<T: Structure>(&self,
                                field_int_map: &FieldIntMap)
                                -> Result<ItemWrapper<T>, Error> {
        Ok(serde_json::from_value::<ItemWrapper<T>>(encoder::decode_wrapper(self, field_int_map))?)
    }
}
