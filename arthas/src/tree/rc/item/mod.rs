
use std::sync::RwLock;
use std::ops::Deref;
use std::ops::DerefMut;
use item::{Id, FieldInt, get_len_field_int};
use serde_json::Value;
use std::collections::HashMap;
use super::RcData;
use super::RcNode;
use traits::get_unique_int_str;
use super::super::len::Len;


lazy_static! {
    static ref _ID_INT_STR: String = get_unique_int_str("_id");
}


#[derive(Clone, Debug)]
pub struct RcItem(*mut RwLock<Item>);

impl RcItem {
    pub fn new(id: Id, value: Value) -> RcItem {
        RcItem(Box::into_raw(Box::new(RwLock::new(Item::new(id, value)))))
    }

    pub fn destroy(self) {
        let rc_item = unsafe { Box::from_raw(self.0) };
        let mut item = rc_item.write().unwrap();
        item.nodes.clear();

        for (_, rc_data) in item.datas.drain() {
            rc_data.destroy();
        }
    }
}

unsafe impl Send for RcItem {}
unsafe impl Sync for RcItem {}

impl Deref for RcItem {
    type Target = RwLock<Item>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for RcItem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}


#[derive(Clone, Debug)]
pub struct Item {
    pub id: Id,
    pub nodes: HashMap<FieldInt, RcNode>,
    pub datas: HashMap<FieldInt, RcData>,
    pub value: Value,
}

impl Item {
    pub fn new(id: Id, value: Value) -> Item {
        if value.is_object() {
            let mut datas = HashMap::new();

            for (field_int, value) in value.get("item").unwrap().as_object().unwrap() {
                if field_int != &*_ID_INT_STR {
                    datas.insert(field_int.to_owned(), RcData::new(value as *const Value));

                    if value.can_len() {
                        datas.insert(get_len_field_int(field_int),
                                     value.create_len_rc_data().unwrap());
                    }
                }
            }

            Item {
                id: id,
                nodes: HashMap::new(),
                datas: datas,
                value: value,
            }
        } else {
            unreachable!()
        }
    }

    pub fn get_pointer(&self) -> *const Value {
        &self.value as *const Value
    }
}
