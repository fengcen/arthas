
use std::fs::File;
use utils::file::open_log_file;
use serde_json::Value;
use to_value;
use item::Id;
use std::io::Write;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Action {
    Insert,
    Delete,
    Clear,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Line {
    pub action: Action,
    pub id: Option<Id>,
    pub value: Option<Value>,
}

impl Line {
    pub fn new(action: Action) -> Line {
        Line {
            action: action,
            id: None,
            value: None,
        }
    }

    pub fn id(mut self, id: Id) -> Line {
        self.id = Some(id);
        self
    }

    pub fn value(mut self, value: Value) -> Line {
        self.value = Some(value);
        self
    }
}

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(struct_name: &str) -> Logger {
        Logger { file: open_log_file(struct_name) }
    }

    pub fn insert(&mut self, id: Id, value: Value) {
        writeln!(self.file,
                 "{}",
                 to_value(Line::new(Action::Insert).id(id).value(value)))
            .unwrap();
    }

    pub fn delete(&mut self, id: Id) {
        writeln!(self.file, "{}", to_value(Line::new(Action::Delete).id(id))).unwrap();
    }

    pub fn clear(&mut self) {
        writeln!(self.file, "{}", to_value(Line::new(Action::Clear))).unwrap();
    }
}
