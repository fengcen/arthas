
mod order;
mod query_type;
mod action;

#[macro_use]
mod macros {
    macro_rules! exec_query {
        ($store:ident, $access:ident, $action:ident, $query: ident) => {
            {
                check_query(&$query)?;

                thread_trace!("{:?}: {}, lock done", $query.action, $query.struct_name);

                let result = $store.get_memory::<T>()
                    .$access()
                    .unwrap().$action(&mut $query);

                thread_trace!("{:?}: {}, query time: {} ms, result: {:?}",
                            $query.action,
                            $query.struct_name,
                            $query.get_query_time(),
                            result);

                result
            }
        }
    }
}

pub use self::order::Order;
pub use self::query_type::QueryType;

use std::sync::RwLockReadGuard;
use std::collections::HashMap;
use std::time::Instant;
use item::{Id, StructName, get_len_field_int};
use memory::Memory;
use traits::Structure;
use store::{MemoryStore, memories, persistences, is_persistence, MemoryGetter};
use std::sync::RwLock;
use persistence::Persistence;
use error::Error;
use std::fmt;
use tree::comparision::Comparision;
use tree::cmp::Cmp;
use to_value;
use serde::Serialize;
use traits::get_unique_int_str;
use self::action::Action;


pub type Updater<'a, T> = Box<Fn(&mut T) + 'a>;

/// Query.
#[derive(Default)]
pub struct Query<'a, T> {
    #[doc(hidden)]
    pub updater: Option<Updater<'a, T>>,
    #[doc(hidden)]
    pub item: Option<T>,
    #[doc(hidden)]
    pub id: Option<Id>,
    #[doc(hidden)]
    pub limit: Option<usize>,
    #[doc(hidden)]
    pub offset: Option<usize>,
    #[doc(hidden)]
    pub field: Option<String>,
    #[doc(hidden)]
    pub len: Option<String>,
    #[doc(hidden)]
    pub conditions: HashMap<String, Vec<Comparision>>,
    #[doc(hidden)]
    pub orders: Vec<(String, Order)>,
    struct_name: StructName,
    action: Action,
    start_time: Option<Instant>,
}

impl<'a, T: Structure> Query<'a, T> {
    /// Create new query.
    pub fn new() -> Query<'a, T> {
        Query { struct_name: T::get_struct_name(), ..Default::default() }
    }

    /// Query with id.
    pub fn id<I: AsRef<str>>(mut self, id: I) -> Query<'a, T> {
        self.id = Some(id.as_ref().into());
        self
    }

    /// Limit like sql.
    pub fn limit(mut self, limit: usize) -> Query<'a, T> {
        self.limit = Some(limit);
        self
    }

    /// Offset like sql.
    pub fn offset(mut self, offset: usize) -> Query<'a, T> {
        self.offset = Some(offset);
        self
    }

    /// Set field for later comparision.
    pub fn field<I: AsRef<str>>(mut self, field: I) -> Query<'a, T> {
        self.field = Some(field.as_ref().into());
        self
    }

    /// This method tests for `self` and `other` values to be equal, and is used by `==`.
    pub fn eq<V: Serialize>(mut self, other: V) -> Query<'a, T> {
        self.compare(Cmp::Eq, other);
        self
    }

    /// This method tests for `!=`.
    pub fn ne<V: Serialize>(mut self, other: V) -> Query<'a, T> {
        self.compare(Cmp::Ne, other);
        self
    }

    /// This method tests greater than (for `self` and `other`) and is used by the `>` operator.
    pub fn gt<V: Serialize>(mut self, other: V) -> Query<'a, T> {
        self.compare(Cmp::Gt, other);
        self
    }

    /// This method tests less than (for `self` and `other`) and is used by the `<` operator.
    pub fn lt<V: Serialize>(mut self, other: V) -> Query<'a, T> {
        self.compare(Cmp::Lt, other);
        self
    }

    /// This method tests greater than or equal to (for `self` and `other`) and is used by the `>=` operator.
    pub fn ge<V: Serialize>(mut self, other: V) -> Query<'a, T> {
        self.compare(Cmp::Ge, other);
        self
    }

    /// This method tests less than or equal to (for `self` and `other`) and is used by the `<=` operator.
    pub fn le<V: Serialize>(mut self, other: V) -> Query<'a, T> {
        self.compare(Cmp::Le, other);
        self
    }

    /// Query the field's length.
    pub fn len<I: AsRef<str>>(mut self, field: I) -> Query<'a, T> {
        self.len = Some(field.as_ref().into());
        self
    }

    /// Query order desc field.
    pub fn desc<I: AsRef<str>>(mut self, field: I) -> Query<'a, T> {
        self.orders.push((get_unique_int_str(field.as_ref()), Order::Desc));
        self
    }

    /// Query order asc field.
    pub fn asc<I: AsRef<str>>(mut self, field: I) -> Query<'a, T> {
        self.orders.push((get_unique_int_str(field.as_ref()), Order::Asc));
        self
    }

    /// Insert an item.
    pub fn insert(mut self, item: T) -> Result<Id, Error> {
        self.item = Some(item);
        let store = self.prepare(Action::Insert)?;
        exec_query!(store, write, insert, self)
    }

    /// Remove item.
    pub fn remove(mut self) -> Result<Vec<T>, Error> {
        let store = self.prepare(Action::Remove)?;
        exec_query!(store, write, delete, self)
    }

    /// Update items.
    pub fn update<F>(mut self, updater: F) -> Result<usize, Error>
        where F: Fn(&mut T) + 'a
    {
        self.updater = Some(Box::new(updater));
        let store = self.prepare(Action::Update)?;
        exec_query!(store, write, update, self)
    }

    /// Replace an item.
    pub fn replace(mut self, item: T) -> Result<(), Error> {
        self.item = Some(item);
        let store = self.prepare(Action::Replace)?;
        exec_query!(store, write, replace, self)
    }

    /// Find items.
    pub fn find(mut self) -> Result<Vec<T>, Error> {
        let store = self.prepare(Action::Find)?;
        exec_query!(store, read, find, self)
    }

    /// Find only one item.
    pub fn find_one(mut self) -> Result<Option<T>, Error> {
        let store = self.prepare(Action::FindOne)?;
        exec_query!(store, read, find_one, self)
    }

    /// Count items.
    pub fn count(mut self) -> Result<usize, Error> {
        let store = self.prepare(Action::Count)?;
        self.orders.clear();
        exec_query!(store, read, count, self)
    }

    #[inline]
    fn prepare<'b>(&mut self, action: Action) -> Result<RwLockReadGuard<'b, MemoryStore>, Error> {
        self.action = action;
        self.start_time = Some(Instant::now());
        thread_trace!("{:?}: {}, query: {:?}, wait for lock.",
                      self.action,
                      self.struct_name,
                      self);
        self.get_memory()
    }

    fn get_memory<'b>(&self) -> Result<RwLockReadGuard<'b, MemoryStore>, Error> {
        let struct_name = T::get_struct_name();
        let store_lock = memories();

        if !store_lock.read().unwrap().contains_key(&struct_name) {
            if is_persistence() {
                let persistence_lock = persistences();
                persistence_lock.write()
                    .unwrap()
                    .entry(struct_name.clone())
                    .or_insert_with(|| {
                        RwLock::new(Persistence::new(struct_name.clone(), T::get_field_int_map()))
                    });
            }

            store_lock.write()
                .unwrap()
                .entry(struct_name.clone())
                .or_insert_with(|| RwLock::new(Memory::new::<T>()));
        }

        Ok(store_lock.read().unwrap())
    }

    #[inline]
    fn get_query_time(&self) -> f64 {
        ((self.start_time.as_ref().unwrap().elapsed().subsec_nanos()) as f64 / 1000.0).round() /
        1000.0
    }

    fn compare<V: Serialize>(&mut self, cmp: Cmp, value: V) {
        let field_int = if self.field.is_some() {
            get_unique_int_str(&self.field.take().unwrap())
        } else if self.len.is_some() {
            get_len_field_int(&get_unique_int_str(&self.len.take().unwrap()))
        } else {
            unreachable!()
        };

        self.conditions
            .entry(field_int.clone())
            .or_insert_with(Vec::new)
            .push(Comparision::new(field_int, cmp, to_value(value)));
    }
}

impl<'a, T: Structure> fmt::Debug for Query<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{{ item: {:?}, id: {:?}, limit: {:?}, field: {:?}, conditions: {:?}, orders: {:?} \
                }}",
               self.item,
               self.id,
               self.limit,
               self.field,
               self.conditions,
               self.orders)
    }
}


#[inline]
fn check_query<T: Structure>(query: &Query<T>) -> Result<(), Error> {
    match query.action {
        Action::Replace => check_replace_query(query),
        _ => Ok(()),
    }
}

#[inline]
fn check_replace_query<T: Structure>(query: &Query<T>) -> Result<(), Error> {
    if !T::has_id() {
        return Err(Error::CanNotReplace);
    }

    if query.item.as_ref().unwrap().get_id().is_empty() && query.id.is_none() {
        return Err(Error::RequiresId);
    }

    Ok(())
}
