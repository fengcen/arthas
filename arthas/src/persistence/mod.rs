
pub mod operation;
pub mod meta;
pub mod service;
pub mod logger;


use std::str;
use std::collections::HashMap;
use item::StructName;
use persistence::operation::Operation;
use traits::FieldIntMap;
use encoder::bin_encode;
use item::Id;
use self::meta::{Meta, SimpleMeta};
use self::logger::{Line, Action};
use serde_json;
use BINENCODE;


#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Persistence {
    pub struct_name: StructName,
    pub field_int_map: FieldIntMap,
    pub metas: HashMap<Id, SimpleMeta>,
    pub deleted: Vec<SimpleMeta>,
    pub size: usize,
}

impl Persistence {
    pub fn new(struct_name: StructName, field_int_map: FieldIntMap) -> Persistence {
        Persistence {
            struct_name: struct_name,
            field_int_map: field_int_map,
            ..Default::default()
        }
    }

    pub fn insert(&mut self, operation: Operation) -> (bool, SimpleMeta, Vec<u8>) {
        let bytes = self.get_bytes(&operation).unwrap();
        let meta = Meta::new(&bytes).to_simple();
        let (append, meta) = self.insert_meta(operation.id.unwrap(), meta);
        (append, meta, bytes)
    }

    pub fn delete(&mut self, operation: Operation) {
        let removed = self.metas.remove(&operation.id.unwrap());
        if removed.is_some() {
            let meta = removed.unwrap();
            self.deleted.push(meta);
        }
    }

    pub fn clear(&mut self) {
        for (_, meta) in self.metas.drain() {
            self.deleted.push(meta);
        }

        self.size = 0;
    }

    pub fn write_log(&mut self, log: String) -> Option<(bool, SimpleMeta, Vec<u8>)> {
        let line = serde_json::from_str::<Line>(&log).unwrap();
        match line.action {
            Action::Insert => {
                Some(self.insert(Operation::new().id(line.id.unwrap()).value(line.value.unwrap())))
            }
            Action::Delete => {
                self.delete(Operation::new().id(line.id.unwrap()).delete());
                None
            }
            Action::Clear => {
                self.clear();
                None
            }
        }
    }

    fn insert_meta(&mut self, id: Id, mut meta: SimpleMeta) -> (bool, SimpleMeta) {
        let mut append = false;
        let return_meta;

        if let Some(mut old_meta) = self.find_in_deleted(meta.size()) {
            old_meta.set_real(meta.real());
            return_meta = old_meta.clone();
            self.metas.insert(id, old_meta);
        } else if self.size > 0 {
            meta.set_offset(self.size);
            return_meta = meta.clone();
            self.metas.insert(id, meta);
            append = true;
        } else {
            return_meta = meta.clone();
            self.metas.insert(id, meta);
            append = true;
        }

        self.size += return_meta.size();

        (append, return_meta)
    }

    fn get_bytes(&self, operation: &Operation) -> Option<Vec<u8>> {
        if operation.value.is_some() {
            let data = operation.value.as_ref().unwrap().to_string();

            Some(if BINENCODE {
                bin_encode(data)
            } else {
                data.as_bytes().to_owned()
            })
        } else {
            None
        }
    }

    fn find_in_deleted(&mut self, size: usize) -> Option<SimpleMeta> {
        let mut index_option = None;

        for (index, meta) in self.deleted.iter().enumerate() {
            if meta.size() >= size {
                index_option = Some(index);
                break;
            }
        }

        if index_option.is_some() {
            Some(self.deleted.swap_remove(index_option.unwrap()))
        } else {
            None
        }
    }
}
