

#[derive(Default, Debug)]
pub struct SearchAction {
    pub take: bool,
    pub fold_left: bool,
    pub fold_right: bool,
    pub go_left: bool,
    pub go_right: bool,
}

impl SearchAction {
    pub fn new() -> SearchAction {
        SearchAction { ..Default::default() }
    }

    pub fn merge(&self, other: &SearchAction) -> Option<SearchAction> {
        if self.go_left != other.go_left || self.go_right != other.go_right {
            return None;
        }

        let mut new_action = SearchAction::new();

        if self.take && other.take {
            new_action.take = true;
        }

        if self.fold_left && other.fold_left {
            new_action.fold_left = true;
        }

        if self.fold_right && other.fold_right {
            new_action.fold_right = true;
        }

        new_action.go_left = self.go_left;
        new_action.go_right = self.go_right;

        Some(new_action)
    }

    pub fn is_stopped(&self) -> bool {
        !self.go_left && !self.go_right
    }

    pub fn take(mut self) -> SearchAction {
        self.take = true;
        self
    }

    pub fn go_right(mut self) -> SearchAction {
        self.go_right = true;
        self
    }

    pub fn go_left(mut self) -> SearchAction {
        self.go_left = true;
        self
    }

    pub fn fold_right(mut self) -> SearchAction {
        self.fold_right = true;
        self
    }

    pub fn fold_left(mut self) -> SearchAction {
        self.fold_left = true;
        self
    }
}
