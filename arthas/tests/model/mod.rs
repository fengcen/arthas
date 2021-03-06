
extern crate serde_json;
extern crate arthas;

pub use arthas::traits::Arthas;
use std::collections::HashMap;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Arthas)]
#[arthas(is_one, rename="string_one=one")]
pub struct Atomic {
    pub string_one: String,
    pub string_two: String,
    pub hash_map: HashMap<String, usize>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Arthas)]
pub struct Comment {
    pub title: String,
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Arthas)]
pub struct Article {
    pub _id: String,
    pub title: String,
    pub content: String,
    pub day_to_views: HashMap<String, usize>,
    pub views: usize,
    pub comments: Vec<Comment>,
}

impl Article {
    pub fn new<T: Into<String>>(title: T) -> Article {
        Article { title: title.into(), ..Default::default() }
    }

    pub fn views(mut self, views: usize) -> Article {
        self.views = views;
        self
    }

    pub fn content<T: Into<String>>(mut self, content: T) -> Article {
        self.content = content.into();
        self
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Arthas)]
pub struct Comments {
    pub day_to_comments: HashMap<String, Comment>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Arthas)]
pub struct Articles {
    pub day_to_articles: HashMap<String, Article>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Arthas)]
pub struct Blog {
    pub articles: Articles,
    pub comments: Comments,
}
