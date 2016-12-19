
use item::Id;
use serde_json::Value;
use error::Error;
use super::logger::Logger;
use store::is_persistence;
use traits::Structure;
use loader::{persistence_exits, create_persistence};
use utils::reflect::get_type_name;


pub struct PersistenceService {
    pub logger: Option<Logger>,
}

impl PersistenceService {
    pub fn new<T: Structure>() -> PersistenceService {
        let struct_name = get_type_name::<T>();

        if is_persistence() && !persistence_exits(&struct_name) {
            create_persistence(&struct_name, &T::get_field_int_map());
        }

        PersistenceService {
            logger: if is_persistence() {
                Some(Logger::new(&struct_name))
            } else {
                None
            },
        }
    }

    pub fn insert(&mut self, id: Id, value: Value) -> Result<(), Error> {
        self.logger.as_mut().unwrap().insert(id, value);
        Ok(())
    }

    pub fn delete(&mut self, id: Id) -> Result<(), Error> {
        self.logger.as_mut().unwrap().delete(id);
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        self.logger.as_mut().unwrap().clear();
        Ok(())
    }
}
