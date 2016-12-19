
quick_error! {
    #[derive(Debug, PartialEq)]
    /// Arthas error.
    pub enum Error {
        /// Only struct which has field `_id` can use `replace` method.
        CanNotReplace {
            display("Only struct which has field `_id` can use replace method")
        }
        /// Serialize error.
        Serialize {
            from(::serde_json::error::Error)
        }
        /// Required id.
        RequiresId {
            display("Required id.")
        }
        /// No field in the struct.
        FieldNotFound {
            display("Query field can not be not found in the struct.")
        }
    }
}
