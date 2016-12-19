
use std::sync::RwLock;
use std::ops::{Deref, DerefMut};
use std::fmt;
use serde_json::Value;
use super::{RcData, RcItem, DataType};


#[derive(Clone)]
pub struct RcChild(*mut RwLock<Child>);

impl RcChild {
    pub fn new(data: RcData, item: RcItem) -> RcChild {
        RcChild(Box::into_raw(Box::new(RwLock::new(Child::new(data, item)))))
    }

    pub fn destroy(self) {
        unsafe {
            Box::from_raw(self.0);
        }
    }
}

unsafe impl Send for RcChild {}
unsafe impl Sync for RcChild {}

impl fmt::Debug for RcChild {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.read().unwrap().get_item_value())
    }
}

impl Deref for RcChild {
    type Target = RwLock<Child>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for RcChild {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}


#[derive(Clone)]
pub struct Child {
    pub data: RcData,
    pub item: RcItem,
}

impl Child {
    pub fn new(data: RcData, item: RcItem) -> Child {
        Child {
            data: data,
            item: item,
        }
    }

    pub fn get_item_pointer(&self) -> *const Value {
        self.item.read().unwrap().get_pointer()
    }

    pub fn get_item_value(&self) -> &Value {
        unsafe { &*self.get_item_pointer() }
    }

    pub fn get_value(&self) -> Value {
        self.data.read().unwrap().get_value().clone()
    }

    pub fn get_type(&self) -> DataType {
        self.data.read().unwrap()._type.clone()
    }
}

impl fmt::Debug for Child {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
