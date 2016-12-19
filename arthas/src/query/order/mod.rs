
#[derive(PartialEq, Debug, Clone)]
pub enum Order {
    Asc,
    Desc,
}

impl Default for Order {
    fn default() -> Order {
        Order::Asc
    }
}
