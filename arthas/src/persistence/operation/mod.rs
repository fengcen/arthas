
use serde_json::Value;
use item::Id;


#[derive(Default)]
pub struct Operation {
    pub value: Option<Value>,
    pub id: Option<Id>,
    pub insert: bool,
    pub delete: bool,
}

impl Operation {
    pub fn new() -> Operation {
        Operation { ..Default::default() }
    }

    pub fn id(mut self, id: Id) -> Operation {
        self.id = Some(id);
        self
    }

    pub fn value(mut self, value: Value) -> Operation {
        self.value = Some(value);
        self
    }

    pub fn insert(mut self) -> Operation {
        self.insert = true;
        self
    }

    pub fn delete(mut self) -> Operation {
        self.delete = true;
        self
    }
}
