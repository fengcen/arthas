
use std::collections::HashSet;
use super::super::comparision::Comparision;
use item::FieldInt;
use query::{Query, Order, QueryType};
use super::super::rc::RcChild;

pub type Subs = Vec<Sub>;
pub type Orders = Vec<(String, Order)>;

#[derive(Debug)]
pub struct Sub {
    pub field_int: FieldInt,
    pub comparisions: Vec<Comparision>,
    pub order: Option<Order>,
    pub stopped: bool,
}

impl Sub {
    pub fn _match(&self, rc_child: &RcChild) -> bool {
        let child = rc_child.read().unwrap();
        let item = child.item.read().unwrap();
        let rc_data = item.datas.get(&self.field_int).unwrap();
        let data = rc_data.read().unwrap();
        let value = data.get_value();

        for comparision in &self.comparisions {
            if !comparision._match(value) {
                return false;
            }
        }
        true
    }
}


#[derive(Default, Debug)]
pub struct Task {
    pub subs: Subs,
    pub orders: Orders,
    pub order_field: Option<String>,
    pub order: Option<Order>,
    pub offset: usize,
    pub limit: Option<usize>,
    pub query_type: QueryType,
    pub subs_length: usize,
    pub current: usize,
}

impl Task {
    pub fn new<T>(query: &Query<T>, query_type: &QueryType) -> Task {
        let (subs_length, subs) = Self::create_subs(query);
        let orders = Self::create_orders(query);
        let order_field = if let Some(order) = orders.first() {
            Some(order.0.to_owned())
        } else {
            None
        };
        let order = if let Some(order) = orders.first() {
            Some(order.1.clone())
        } else {
            None
        };

        Task {
            subs: subs,
            orders: orders,
            order: order,
            offset: query.offset.unwrap_or(0),
            limit: query.limit.clone(),
            query_type: query_type.clone(),
            subs_length: subs_length,
            order_field: order_field,
            current: 0,
        }
    }

    fn create_orders<T>(query: &Query<T>) -> Orders {
        let mut fields = HashSet::new();
        let mut orders = Vec::new();

        for tup in &query.orders {
            if fields.contains(&tup.0) {
                continue;
            } else {
                orders.push(tup.clone());
                fields.insert(tup.0.clone());
            }
        }

        orders
    }

    fn create_subs<T>(query: &Query<T>) -> (usize, Subs) {
        let mut subs_length = 0;
        let mut subs = Vec::new();

        for (field_int, comparisions) in &query.conditions {
            subs_length += 1;
            let mut sub_order = None;

            if !query.orders.is_empty() {
                let order = query.orders.first().unwrap();
                if field_int == &order.0 {
                    sub_order = Some(order.1.clone());
                }
            }

            subs.push(Sub {
                field_int: field_int.to_owned(),
                comparisions: comparisions.clone(),
                order: sub_order,
                stopped: false,
            });
        }

        (subs_length, subs)
    }

    pub fn get_order_field(&self) -> &str {
        self.order_field.as_ref().unwrap()
    }

    pub fn has_order(&self) -> bool {
        self.order_field.is_some()
    }
}
