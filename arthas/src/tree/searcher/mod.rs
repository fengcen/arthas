
pub mod task;
pub mod executor;
pub mod meet;
pub mod entrance_type;

use super::Tree;
use query::{Query, QueryType};
use error::Error;
use serde_json::Value;
use traits::Structure;
use self::task::Task;
use self::executor::Exectuor;
use scoped_pool::Pool;


pub struct Searcher {}

impl Searcher {
    pub fn new() -> Searcher {
        Searcher {}
    }

    pub fn search<T: Structure>(&self,
                                pool: &Pool,
                                tree: &Tree,
                                query: &Query<T>,
                                query_type: &QueryType)
                                -> Result<(usize, Vec<*const Value>), Error> {
        if query.id.is_some() {
            thread_trace!("search by id: {:?}", query.id);
            self.search_by_id(tree, query, query_type)
        } else if !query.conditions.is_empty() {
            thread_trace!("search by conditions.");
            self.search_by_query(pool, tree, query, query_type)
        } else {
            thread_trace!("search all.");
            self.search_all(tree, query, query_type)
        }
    }

    fn search_by_id<T: Structure>(&self,
                                  tree: &Tree,
                                  query: &Query<T>,
                                  query_type: &QueryType)
                                  -> Result<(usize, Vec<*const Value>), Error> {
        let mut values = Vec::new();
        let mut count = 0;

        if let Some(rc_item) = tree.id_map.get(query.id.as_ref().unwrap()) {
            if query.offset.is_none() || *query.offset.as_ref().unwrap() == 0 {
                count += 1;

                if query_type == &QueryType::Find {
                    values.push(rc_item.read().unwrap().get_pointer());
                }
            }
        }

        Ok((count, values))
    }

    fn search_by_query<T: Structure>(&self,
                                     pool: &Pool,
                                     tree: &Tree,
                                     query: &Query<T>,
                                     query_type: &QueryType)
                                     -> Result<(usize, Vec<*const Value>), Error> {
        Exectuor::exec(pool, tree, Task::new(query, query_type))
    }

    fn search_all<T: Structure>(&self,
                                tree: &Tree,
                                query: &Query<T>,
                                query_type: &QueryType)
                                -> Result<(usize, Vec<*const Value>), Error> {
        let mut values = Vec::new();
        let mut count = 0;

        thread_trace!("current id map length: {}", tree.id_map.len());

        for (index, rc_item) in tree.id_map.values().enumerate() {
            if query.offset.is_some() && index + 1 < *query.offset.as_ref().unwrap() {
                continue;
            }

            count += 1;

            if query_type == &QueryType::Find {
                values.push(rc_item.read().unwrap().get_pointer());
            }

            if query.limit.is_some() {
                if count >= *query.limit.as_ref().unwrap() {
                    break;
                }
            }

        }

        Ok((count, values))
    }
}
