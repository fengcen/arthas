
use std::cmp::Ordering;
use serde_json::Value;
use query::Order;


pub trait Math {
    fn eq(&self, &Value) -> bool;
    fn ne(&self, &Value) -> bool;
    fn gt(&self, &Value) -> bool;
    fn lt(&self, &Value) -> bool;
    fn ge(&self, &Value) -> bool;
    fn le(&self, &Value) -> bool;
    fn cmp(&self, &Value, order: &Order) -> Ordering;
    fn get_rate(&self, &Value) -> f64;
}

impl Math for Value {
    fn eq(&self, value: &Value) -> bool {
        thread_trace!("math eq, current: {:?}, find: {:?}", self, value);

        if self.is_number() && value.is_number() {
            self.get_f64() == value.get_f64()
        } else {
            self == value
        }
    }

    fn ne(&self, value: &Value) -> bool {
        thread_trace!("math ne, current: {:?}, find: {:?}", self, value);

        if self.is_number() && value.is_number() {
            self.get_f64() != value.get_f64()
        } else {
            self != value
        }
    }

    fn gt(&self, value: &Value) -> bool {
        thread_trace!("math gt, current: {:?}, find: {:?}", self, value);

        if self.is_number() && value.is_number() {
            self.get_f64() > value.get_f64()
        } else if self.is_string() && value.is_string() {
            self.get_str() > value.get_str()
        } else {
            panic!("Unsupported math type");
        }
    }

    fn lt(&self, value: &Value) -> bool {
        thread_trace!("math lt, current: {:?}, find: {:?}", self, value);

        if self.is_number() && value.is_number() {
            self.get_f64() < value.get_f64()
        } else if self.is_string() && value.is_string() {
            self.get_str() < value.get_str()
        } else {
            panic!("Unsupported math type");
        }
    }

    fn ge(&self, value: &Value) -> bool {
        thread_trace!("math ge, current: {:?}, find: {:?}", self, value);

        if self.is_number() && value.is_number() {
            self.get_f64() >= value.get_f64()
        } else if self.is_string() && value.is_string() {
            self.get_str() >= value.get_str()
        } else {
            panic!("Unsupported math type");
        }
    }

    fn le(&self, value: &Value) -> bool {
        thread_trace!("math le, current: {:?}, find: {:?}", self, value);

        if self.is_number() && value.is_number() {
            self.get_f64() <= value.get_f64()
        } else if self.is_string() && value.is_string() {
            self.get_str() <= value.get_str()
        } else {
            panic!("Unsupported math type");
        }
    }

    fn cmp(&self, value: &Value, order: &Order) -> Ordering {
        thread_trace!("math cmp, current: {:?}, find: {:?}", self, value);

        if let Order::Desc = *order {
            if Math::eq(self, value) {
                Ordering::Equal
            } else if Math::lt(self, value) {
                Ordering::Greater
            } else if Math::gt(self, value) {
                Ordering::Less
            } else {
                panic!("Unsupported math type");
            }
        } else {
            if Math::eq(self, value) {
                Ordering::Equal
            } else if Math::lt(self, value) {
                Ordering::Less
            } else if Math::gt(self, value) {
                Ordering::Greater
            } else {
                panic!("Unsupported math type");
            }
        }
    }

    fn get_rate(&self, other: &Value) -> f64 {
        if self.is_number() && other.is_number() {
            let n1 = self.get_f64();
            let n2 = other.get_f64();
            n2 / n1
        } else if self.is_string() && other.is_string() {
            let n1 = self.get_str().chars().map(|c| c as usize).fold(0, |a, b| a + b);
            let n2 = other.get_str().chars().map(|c| c as usize).fold(0, |a, b| a + b);
            n2 as f64 / n1 as f64
        } else {
            panic!("Unsupported math type");
        }
    }
}


trait Type {
    fn get_f64(&self) -> f64;
    fn get_string(&self) -> String;
    fn get_str(&self) -> &str;
    fn get_u64(&self) -> u64;
    fn get_i64(&self) -> i64;
    fn get_boolean(&self) -> bool;
    fn format(&self) -> String;
}

impl Type for Value {
    fn get_f64(&self) -> f64 {
        match *self {
            Value::F64(f) => f,
            Value::I64(f) => f as f64,
            Value::U64(f) => f as f64,
            _ => panic!("not a number"),
        }
    }

    fn get_string(&self) -> String {
        self.as_str().unwrap().to_owned()
    }

    fn get_str(&self) -> &str {
        self.as_str().unwrap()
    }

    fn get_u64(&self) -> u64 {
        self.as_u64().unwrap()
    }

    fn get_i64(&self) -> i64 {
        self.as_i64().unwrap()
    }

    fn get_boolean(&self) -> bool {
        self.as_bool().unwrap()
    }

    fn format(&self) -> String {
        format!("{:?}", self)
    }
}
