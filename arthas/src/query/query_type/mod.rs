
#[derive(PartialEq, Debug, Clone)]
pub enum QueryType {
    Find,
    Count,
}

impl Default for QueryType {
    fn default() -> QueryType {
        QueryType::Find
    }
}
