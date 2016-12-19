
#[derive(Debug)]
pub enum Action {
    Insert,
    Find,
    FindOne,
    Count,
    Replace,
    Update,
    Remove,
}

impl Default for Action {
    fn default() -> Action {
        Action::Find
    }
}
