
use std::cmp::Ordering;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::collections::HashMap;
use super::task::Task;
use error::Error;
use super::Tree;
use super::super::rc::RcChild;
use item::FieldInt;
use serde_json::Value;
use query::QueryType;
use super::super::node::{Group, Groups};
use quickersort::sort_by;
use super::super::math::Math;
use super::task::{Sub, Orders};
use scoped_pool::Pool;
use super::entrance_type::EntranceType;


pub struct Exectuor {}

impl Exectuor {
    pub fn exec(pool: &Pool,
                tree: &Tree,
                mut task: Task)
                -> Result<(usize, Vec<*const Value>), Error> {
        if tree.id_map.is_empty() {
            return Ok(Default::default());
        }

        let field_groups = Mutex::new(HashMap::new());
        let field_sub = Mutex::new(HashMap::new());
        let stopped = AtomicBool::new(false);

        pool.scoped(|scope| {
            let field_groups = &field_groups;
            let field_sub = &field_sub;
            let stopped = &stopped;

            for mut sub in task.subs.drain(..) {
                let entrance_type = EntranceType::new(tree,
                                                      &sub.field_int,
                                                      &sub.comparisions.first().unwrap().other);

                match entrance_type {
                    EntranceType::Root => {
                        scope.execute(move || {
                            thread_trace!("search root");

                            let mut groups = Groups::new();
                            tree.root
                                .get(&sub.field_int)
                                .unwrap()
                                .read()
                                .unwrap()
                                .search_root(stopped, &mut groups, &mut sub);

                            field_groups.lock().unwrap().insert(sub.field_int.clone(), groups);
                            field_sub.lock().unwrap().insert(sub.field_int.clone(), sub);
                        });
                    }
                    EntranceType::Min => {
                        scope.execute(move || {
                            let mut groups = Groups::new();
                            let rc_node = tree.min
                                .get(&sub.field_int)
                                .unwrap();

                            thread_trace!("search min, min is {:?}",
                                          rc_node.read().unwrap().get_value());

                            rc_node.read()
                                .unwrap()
                                .search_min(stopped, &mut groups, &mut sub);

                            field_groups.lock().unwrap().insert(sub.field_int.clone(), groups);
                            field_sub.lock().unwrap().insert(sub.field_int.clone(), sub);
                        })
                    }
                    EntranceType::Max => {
                        scope.execute(move || {
                            let mut groups = Groups::new();
                            let rc_node = tree.max
                                .get(&sub.field_int)
                                .unwrap();

                            thread_trace!("search max, max is {:?}",
                                          rc_node.read().unwrap().get_value());

                            rc_node.read()
                                .unwrap()
                                .search_max(stopped, &mut groups, &mut sub);

                            field_groups.lock().unwrap().insert(sub.field_int.clone(), groups);
                            field_sub.lock().unwrap().insert(sub.field_int.clone(), sub);
                        })
                    }
                    EntranceType::None => {
                        thread_trace!("no root found!");
                    }
                }
            }
        });

        let mut field_groups = field_groups.into_inner().unwrap();
        let mut field_sub = field_sub.into_inner().unwrap();
        let need_sort = task.has_order();
        let stopped_field = get_stopped_field(&field_sub, &task);

        if field_groups.contains_key(&stopped_field) {
            field_sub.remove(&stopped_field);
            let groups = field_groups.remove(&stopped_field).unwrap();

            if need_sort {
                let mut children = groups_to_children(groups);
                sort_children(&mut children[..], &task.orders);
                return Ok(filter_children(children, &task, &field_sub));
            } else {
                return Ok(filter_groups(groups, &task, &field_sub));
            }
        }

        Ok(Default::default())
    }
}

#[inline]
fn filter_groups(groups: Groups,
                 task: &Task,
                 field_sub: &HashMap<FieldInt, Sub>)
                 -> (usize, Vec<*const Value>) {
    let mut found = 0;
    let mut count = 0;
    let mut values = Vec::new();
    let other_conditions_exists = !field_sub.is_empty();
    let is_count = task.query_type == QueryType::Count;

    'outer: for group in groups {
        if other_conditions_exists {
            for rc_child in group.read().unwrap().values() {
                if filter_child(rc_child,
                                &mut found,
                                &mut count,
                                &mut values,
                                task,
                                field_sub) {
                    break 'outer;
                }
            }
        } else if is_count {
            count += group.read().unwrap().len();
            found = count;
        } else {
            if collect_child(&mut found, &mut count, task, &group, &mut values) {
                break;
            }
        }
    }

    if is_count {
        if count > task.offset {
            count -= task.offset;
        } else {
            count = 0;
        }
    }

    (count, values)
}

#[inline]
fn collect_child(found: &mut usize,
                 count: &mut usize,
                 task: &Task,
                 group: &Group,
                 values: &mut Vec<*const Value>)
                 -> bool {
    for rc_child in group.read().unwrap().values() {
        *found += 1;

        if *found > task.offset {
            *count += 1;
            values.push(rc_child.read().unwrap().get_item_pointer());

            if task.limit.is_some() {
                if *count >= *task.limit.as_ref().unwrap() {
                    return true;
                }
            }
        }
    }

    false
}

#[inline]
fn filter_children(children: Vec<RcChild>,
                   task: &Task,
                   field_sub: &HashMap<FieldInt, Sub>)
                   -> (usize, Vec<*const Value>) {
    let mut found = 0;
    let mut count = 0;
    let mut values = Vec::new();

    for rc_child in children {
        if filter_child(&rc_child,
                        &mut found,
                        &mut count,
                        &mut values,
                        task,
                        field_sub) {
            break;
        }
    }

    (count, values)
}

#[inline]
fn filter_child(rc_child: &RcChild,
                found: &mut usize,
                count: &mut usize,
                values: &mut Vec<*const Value>,
                task: &Task,
                field_sub: &HashMap<FieldInt, Sub>)
                -> bool {
    let mut pass = true;

    for sub in field_sub.values() {
        if !sub._match(&rc_child) {
            thread_trace!("not match, left: {:?}, right: {:?}",
                          rc_child,
                          sub.comparisions);

            pass = false;
            break;
        }
    }

    if pass {
        *found += 1;

        if *found > task.offset {
            *count += 1;
            values.push(rc_child.read().unwrap().get_item_pointer());
        }

        if task.limit.is_some() {
            if *count >= *task.limit.as_ref().unwrap() {
                return true;
            }
        }
    }

    false
}

#[inline]
fn sort_children(children: &mut [RcChild], orders: &Orders) {
    sort_by(children,
            &|a, b| {
        let a_rc_item = a.read().unwrap();
        let a_item = a_rc_item.item.read().unwrap();

        let b_rc_item = b.read().unwrap();
        let b_item = b_rc_item.item.read().unwrap();

        let mut ordering = Ordering::Equal;

        for &(ref field, ref order) in orders {
            let a_rc_data = &a_item.datas[field];
            let a_data = a_rc_data.read().unwrap();
            let a_value = a_data.get_value();


            let b_rc_data = &b_item.datas[field];
            let b_data = b_rc_data.read().unwrap();
            let b_value = b_data.get_value();

            ordering = a_value.cmp(b_value, order);

            if ordering != Ordering::Equal {
                break;
            }
        }

        ordering
    });
}

#[inline]
fn groups_to_children(groups: Vec<Group>) -> Vec<RcChild> {
    let mut children = Vec::new();
    for group in groups {
        for rc_child in group.read().unwrap().values() {
            children.push(rc_child.clone());
        }
    }
    children
}

#[inline]
fn get_stopped_field(field_sub: &HashMap<FieldInt, Sub>, task: &Task) -> String {
    if task.has_order() && field_sub.contains_key(task.get_order_field()) &&
       field_sub.get(task.get_order_field()).unwrap().stopped {
        return task.get_order_field().to_owned();
    }

    for (field, sub) in field_sub {
        if sub.stopped {
            return field.to_owned();
        }
    }

    unreachable!()
}
