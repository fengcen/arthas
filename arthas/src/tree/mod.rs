
pub mod node;
pub mod rc;
pub mod inserter;
pub mod searcher;
pub mod deleter;
pub mod math;
pub mod comparision;
pub mod cmp;
pub mod action;
pub mod len;

use std::collections::HashMap;
use serde_json::Value;
use traits::{Structure, FieldIntMap};
use query::{Query, QueryType};
use error::Error;
use item::{Id, FieldInt};
use self::rc::{RcNode, RcItem};
use self::inserter::Inserter;
use self::searcher::Searcher;
use self::deleter::Deleter;
use utils::hash_map::revert;
use scoped_pool::Pool;


pub struct Tree {
    pub int_field_map: FieldIntMap,
    pub id_map: HashMap<Id, RcItem>,
    pub root: HashMap<FieldInt, RcNode>,
    pub min: HashMap<FieldInt, RcNode>,
    pub max: HashMap<FieldInt, RcNode>,
    pub searcher: Searcher,
}

impl Tree {
    pub fn new<T: Structure>() -> Tree {
        Tree {
            int_field_map: revert(T::get_field_int_map()),
            id_map: HashMap::new(),
            root: HashMap::new(),
            min: HashMap::new(),
            max: HashMap::new(),
            searcher: Searcher::new(),
        }
    }

    pub fn insert(&mut self, id: Id, value: Value) {
        Inserter::insert(self, id, value);
    }

    pub fn delete(&mut self, id: &str) {
        Deleter::delete(self, id);
    }

    pub fn clear(&mut self) {
        Deleter::clear(self);
    }

    pub fn search<T: Structure>(&self,
                                pool: &Pool,
                                query: &Query<T>,
                                query_type: &QueryType)
                                -> Result<(usize, Vec<*const Value>), Error> {
        self.searcher.search(pool, self, query, query_type)
    }
}
