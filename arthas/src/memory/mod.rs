
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use query::Query;
use item::{ItemWrapper, Id, get_len_field_int};
use serde_json::{to_value, Value};
use store;
use traits::{Structure, FieldIntMap};
use persistence::service::PersistenceService;
use error::Error;
use item::convertor::Convertor;
use encoder;
use tree::Tree;
use query::QueryType;
use num_cpus;
use utils::hash_map::revert;
use scoped_pool::Pool;
use vec_map::VecMap;


pub struct Memory {
    pub field_int_map: Arc<FieldIntMap>,
    pub int_field_map: Arc<FieldIntMap>,
    pub service: PersistenceService,
    pub tree: Tree,
    pub is_one: bool,
    pub pool: Pool,
}

impl Memory {
    pub fn new<T: Structure>() -> Memory {
        Memory {
            field_int_map: Arc::new(T::get_field_int_map()),
            int_field_map: Arc::new(revert(T::get_field_int_map())),
            service: PersistenceService::new::<T>(),
            tree: Tree::new::<T>(),
            is_one: T::is_one(),
            pool: Pool::new(num_cpus::get()),
        }
    }

    pub fn insert<T: Structure>(&mut self, query: &mut Query<T>) -> Result<Id, Error> {
        self.check_query_field(query)?;

        let wrapper = ItemWrapper::new(query.item.take().unwrap());
        let id = wrapper.id.clone();
        let encoded = encoder::encode_wrapper(&to_value(wrapper), &self.field_int_map);
        self.insert_encoded_value(id.clone(), encoded)?;
        Ok(id)
    }

    pub fn find<T: Structure>(&self, query: &mut Query<T>) -> Result<Vec<T>, Error> {
        self.check_query_field(query)?;

        Ok(self.search(&query, &QueryType::Find)?
            .1
            .into_iter()
            .map(|wrapper| wrapper.item)
            .collect::<Vec<_>>())
    }

    pub fn count<T: Structure>(&self, query: &mut Query<T>) -> Result<usize, Error> {
        self.check_query_field(query)?;

        Ok(self.search(&query, &QueryType::Count)?.0)
    }

    pub fn find_one<T: Structure>(&self, query: &mut Query<T>) -> Result<Option<T>, Error> {
        self.check_query_field(query)?;

        query.limit = Some(1);
        let mut items = self.find(query)?;

        if !items.is_empty() {
            Ok(Some(items.remove(0)))
        } else if T::is_one() {
            Ok(Some(T::new_empty()))
        } else {
            Ok(None)
        }
    }

    pub fn update<T: Structure>(&mut self, query: &mut Query<T>) -> Result<usize, Error> {
        self.check_query_field(query)?;

        let updater = query.updater.take().unwrap();
        let mut wrappers = self.delete_to_wrapper(query)?;
        let count = wrappers.len();

        for wrapper in &mut wrappers {
            updater(&mut wrapper.item);
        }

        for wrapper in &wrappers {
            let encoded = encoder::encode_wrapper(&to_value(wrapper), &self.field_int_map);
            self.tree.insert(wrapper.id.clone(), encoded);
        }

        if store::is_persistence() {
            for wrapper in wrappers {
                self.service.insert(wrapper.id.clone(), to_value(&wrapper))?;
            }
        }

        Ok(count)
    }

    pub fn replace<T: Structure>(&mut self, query: &mut Query<T>) -> Result<(), Error> {
        self.check_query_field(query)?;

        if query.id.is_some() {
            query.item.as_mut().unwrap().set_id(query.id.clone().unwrap());
        }

        let wrapper = ItemWrapper::new(query.item.clone().unwrap());
        self.delete_to_wrapper::<T>(&mut Query::new().id(wrapper.id.clone()))?;
        self.insert(query)?;

        Ok(())
    }

    pub fn delete<T: Structure>(&mut self, query: &mut Query<T>) -> Result<Vec<T>, Error> {
        self.check_query_field(query)?;

        Ok(self.delete_to_wrapper(query)?
            .into_iter()
            .map(|wrapper| wrapper.item)
            .collect::<Vec<_>>())
    }

    pub fn insert_encoded_value(&mut self, id: Id, value: Value) -> Result<(), Error> {
        if self.is_one {
            self.clear()?;
            self.tree.insert(id.clone(), value.clone());

            if store::is_persistence() {
                self.service.clear()?;
                self.service.insert(id.clone(), value.clone())?;
            }
        } else {
            self.tree.insert(id.clone(), value.clone());

            if store::is_persistence() {
                self.service.insert(id.clone(), value.clone())?;
            }
        }

        Ok(())
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.tree.clear();

        if store::is_persistence() {
            self.service.clear()?;
        }

        Ok(())
    }

    fn delete_to_wrapper<T: Structure>(&mut self,
                                       query: &mut Query<T>)
                                       -> Result<Vec<ItemWrapper<T>>, Error> {
        let wrappers = self.search::<T>(&query, &QueryType::Find)?.1;

        for wrapper in &wrappers {
            self.tree.delete(&wrapper.id);

            if store::is_persistence() {
                self.service.delete(wrapper.id.clone())?;
            }
        }

        Ok(wrappers)
    }

    fn search<T: Structure>(&self,
                            query: &&mut Query<T>,
                            query_type: &QueryType)
                            -> Result<(usize, Vec<ItemWrapper<T>>), Error> {
        let (count, values) = self.tree.search(&self.pool, query, query_type)?;
        let wrappers = if query_type == &QueryType::Find && count > 0 {
            let wrapper_map = Mutex::new(VecMap::new());
            self.pool.scoped(|scope| {
                let wrapper_map = &wrapper_map;
                let int_field_map = &self.int_field_map;

                for (i, value) in values.into_iter().enumerate() {
                    let ptr = AtomicPtr::new(value as *mut Value);

                    scope.execute(move || {
                        let wrapper = unsafe { &*ptr.load(Ordering::SeqCst) }
                            .to_wrapper::<T>(&*int_field_map)
                            .unwrap();
                        wrapper_map.lock().unwrap().insert(i, wrapper);
                    })
                }
            });

            let mut wrappers = Vec::new();
            let mut wrapper_map = wrapper_map.into_inner().unwrap();

            for i in 0..count {
                wrappers.push(wrapper_map.remove(i).unwrap());
            }

            wrappers

        } else {
            Vec::new()
        };

        Ok((count, wrappers))
    }

    #[inline]
    fn check_query_field<T>(&self, query: &Query<T>) -> Result<(), Error> {
        for field in &query.conditions {
            if !self.int_field_map.contains_key(field.0) {
                let mut found = false;

                for field_int in self.int_field_map.keys() {
                    if get_len_field_int(field_int) == *field.0 {
                        found = true;
                        break;
                    }
                }

                if !found {
                    return Err(Error::FieldNotFound);
                }
            }
        }

        Ok(())
    }
}
