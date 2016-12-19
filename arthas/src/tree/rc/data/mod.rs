
use std::sync::RwLock;
use std::ops::Deref;
use std::ops::DerefMut;
use serde_json::Value;
use super::DataType;


#[derive(Clone, Debug)]
pub struct RcData(*mut RwLock<Data>);

impl RcData {
    pub fn new(value: *const Value) -> RcData {
        RcData(Box::into_raw(Box::new(RwLock::new(Data::new(value)))))
    }

    pub fn destroy(self) {
        unsafe {
            Box::from_raw(self.0);
        }
    }
}

unsafe impl Send for RcData {}
unsafe impl Sync for RcData {}

impl Deref for RcData {
    type Target = RwLock<Data>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for RcData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}


#[derive(Clone, Debug)]
pub struct Data {
    pub value: *const Value,
    pub _type: DataType,
}

impl Data {
    pub fn new(value: *const Value) -> Data {
        let _type = DataType::from(unsafe { &*value });

        Data {
            value: value,
            _type: _type,
        }
    }

    pub fn can_index(&self) -> bool {
        match self._type {
            DataType::Number | DataType::String => true,
            _ => false,
        }
    }

    pub fn get_value(&self) -> &Value {
        unsafe { &*self.value }
    }
}
