
pub mod convertor;

use objectid::ObjectId;
use traits::Structure;


/// Item id
pub type Id = String;
pub type StructName = String;
pub type FieldInt = String;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemWrapper<T> {
    pub item: T,
    pub id: Id,
}

impl<T: Structure> ItemWrapper<T> {
    pub fn new(mut item: T) -> ItemWrapper<T> {
        let id = if T::has_id() {
            let mut id = item.get_id();
            if id.is_empty() {
                id = create_id();
                item.set_id(id.clone());
            }
            id
        } else {
            create_id()
        };

        ItemWrapper {
            item: item,
            id: id,
        }
    }
}

#[inline]
fn create_id() -> String {
    ObjectId::new().unwrap().to_string()
}

#[inline]
pub fn get_len_field_int(field_int: &str) -> String {
    format!("len({})", field_int)
}
