#![feature(plugin, custom_derive, test)]
#![plugin(arthas_plugin)]

#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate test;
extern crate arthas;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde_json;

pub mod model;
pub mod common;

use std::collections::HashMap;
use serde_json::to_value;
use arthas::encoder::{encode, decode};
use arthas::traits::get_unique_int_str;
use common::revert;
use model::*;


#[bench]
fn bench_encode(b: &mut test::Bencher) {
    let title = "This is title".to_owned();
    let content = "This is content".to_owned();
    let comment_title = "This is comment title".to_owned();
    let comment_content = "This is comment content".to_owned();
    let field_int_map = Article::get_field_int_map();

    let value = to_value(Article {
        _id: String::new(),
        title: title.clone(),
        content: content.clone(),
        day_to_views: HashMap::new(),
        views: 0,
        comments: vec![Comment {
                           title: comment_title.clone(),
                           content: comment_content.clone(),
                       }],
    });

    b.iter(|| encode(&value, &field_int_map))
}

#[bench]
fn bench_decode(b: &mut test::Bencher) {
    let title = "This is title".to_owned();
    let content = "This is content".to_owned();
    let comment_title = "This is comment title".to_owned();
    let comment_content = "This is comment content".to_owned();
    let int_field_map = revert(Article::get_field_int_map());

    let value = to_value(hashmap!{
            get_unique_int_str("_id") => to_value(""),
            get_unique_int_str("title") => to_value(title.clone()),
            get_unique_int_str("content") => to_value(content.clone()),
            get_unique_int_str("day_to_views") => to_value(HashMap::<String, usize>::new()),
            get_unique_int_str("views") => to_value(0),
            get_unique_int_str("comments.[].title") => to_value(vec![comment_title.clone()]),
            get_unique_int_str("comments.[].content") => to_value(vec![comment_content.clone()])
        });

    b.iter(|| decode(&value, &int_field_map))
}

#[test]
fn test_atomic() {
    let string_one = "This is string one".to_owned();
    let string_two = "This is string two".to_owned();

    let decoded = to_value(Atomic {
        string_one: string_one.clone(),
        string_two: string_two.clone(),
        hash_map: HashMap::new(),
    });

    let encoded = to_value(hashmap!{
            get_unique_int_str("string_one") => to_value(string_one.clone()),
            get_unique_int_str("string_two") => to_value(string_two.clone()),
            get_unique_int_str("hash_map") => to_value(HashMap::<String, usize>::new())
        });

    assert_eq!(encode(&decoded, &Atomic::get_field_int_map()), encoded);
    assert_eq!(decode(&encoded, &revert(Atomic::get_field_int_map())),
               decoded);
}

#[test]
fn test_vec() {
    let title = "This is title".to_owned();
    let content = "This is content".to_owned();
    let comment_title = "This is comment title".to_owned();
    let comment_content = "This is comment content".to_owned();

    let decoded = to_value(Article {
        _id: String::new(),
        title: title.clone(),
        content: content.clone(),
        day_to_views: HashMap::new(),
        views: 0,
        comments: vec![Comment {
                           title: comment_title.clone(),
                           content: comment_content.clone(),
                       }],
    });

    let encoded = to_value(hashmap!{
            get_unique_int_str("_id") => to_value(""),
            get_unique_int_str("title") => to_value(title.clone()),
            get_unique_int_str("content") => to_value(content.clone()),
            get_unique_int_str("day_to_views") => to_value(HashMap::<String, usize>::new()),
            get_unique_int_str("views") => to_value(0),
            get_unique_int_str("comments.[].title") => to_value(vec![comment_title.clone()]),
            get_unique_int_str("comments.[].content") => to_value(vec![comment_content.clone()])
        });

    assert_eq!(encode(&decoded, &Article::get_field_int_map()), encoded);
    assert_eq!(decode(&encoded, &revert(Article::get_field_int_map())),
               decoded);
}

#[test]
fn test_hashmap() {
    let day_26 = "2016-10-26".to_owned();
    let day_27 = "2016-10-27".to_owned();
    let title_26 = "This is day 26 title".to_owned();
    let title_27 = "This is day 27 title".to_owned();
    let content_26 = "This is day 26 content".to_owned();
    let content_27 = "This is day 27 content".to_owned();

    let decoded = to_value(Comments {
        day_to_comments: hashmap!{
                day_26.clone() => Comment {
                    title: title_26.clone(),
                    content: content_26.clone()
                },
                day_27.clone() => Comment {
                    title: title_27.clone(),
                    content: content_27.clone()
                }
            },
    });

    let encoded = to_value(hashmap!{
            get_unique_int_str("day_to_comments.{}.title") => to_value(hashmap!{
                day_26.clone() => title_26.clone(),
                day_27.clone() => title_27.clone()
            }),
            get_unique_int_str("day_to_comments.{}.content") => to_value(hashmap!{
                day_26.clone() => content_26.clone(),
                day_27.clone() => content_27.clone()
            })
        });

    assert_eq!(encode(&decoded, &Comments::get_field_int_map()), encoded);
    assert_eq!(decode(&encoded, &revert(Comments::get_field_int_map())),
               decoded);
}

#[test]
fn test_blog() {
    let comment_title = "This is comment title".to_owned();
    let comment_content = "This is comment content".to_owned();
    let day_26 = "2016-10-26".to_owned();
    let day_27 = "2016-10-27".to_owned();
    let title_26 = "This is day 26 title".to_owned();
    let title_27 = "This is day 27 title".to_owned();
    let content_26 = "This is day 26 content".to_owned();
    let content_27 = "This is day 27 content".to_owned();
    let comment_title_26 = "This is day 26 comment title".to_owned();
    let comment_title_27 = "This is day 27 comment title".to_owned();
    let comment_content_26 = "This is day 26 comment content".to_owned();
    let comment_content_27 = "This is day 27 comment content".to_owned();

    let decoded = to_value(Blog {
        articles: Articles {
            day_to_articles: hashmap!{
                    day_26.clone() => Article {
                        _id: String::new(),
                        title: title_26.clone(),
                        content: content_26.clone(),
                        day_to_views: HashMap::new(),
                        views: 0,
                        comments: vec![Comment {
                            title: comment_title.clone(),
                            content: comment_content.clone()
                        }]
                    },
                    day_27.clone() => Article {
                        _id: String::new(),
                        title: title_27.clone(),
                        content: content_27.clone(),
                        day_to_views: HashMap::new(),
                        views: 0,
                        comments: vec![Comment {
                            title: comment_title.clone(),
                            content: comment_content.clone()
                        }]
                    }
                },
        },
        comments: Comments {
            day_to_comments: hashmap!{
                    day_26.clone() => Comment {
                        title: comment_title_26.clone(),
                        content: comment_content_26.clone()
                    },
                    day_27.clone() => Comment {
                        title: comment_title_27.clone(),
                        content: comment_content_27.clone()
                    }
                },
        },
    });

    let encoded = to_value(hashmap!{
            get_unique_int_str("articles.day_to_articles.{}._id") => to_value(hashmap!{
                day_26.clone() => String::new(),
                day_27.clone() => String::new()
            }),
            get_unique_int_str("articles.day_to_articles.{}.title") => to_value(hashmap!{
                day_26.clone() => title_26.clone(),
                day_27.clone() => title_27.clone()
            }),
            get_unique_int_str("articles.day_to_articles.{}.content") => to_value(hashmap!{
                day_26.clone() => content_26.clone(),
                day_27.clone() => content_27.clone()
            }),
            get_unique_int_str("articles.day_to_articles.{}.day_to_views") => to_value(hashmap!{
                day_26.clone() => HashMap::<String, usize>::new(),
                day_27.clone() => HashMap::<String, usize>::new()
            }),
            get_unique_int_str("articles.day_to_articles.{}.views") => to_value(hashmap!{
                day_26.clone() => 0,
                day_27.clone() => 0
            }),
            get_unique_int_str("articles.day_to_articles.{}.comments.[].title") => to_value(hashmap!{
                day_26.clone() => vec![comment_title.clone()],
                day_27.clone() => vec![comment_title.clone()]
            }),
            get_unique_int_str("articles.day_to_articles.{}.comments.[].content") => to_value(hashmap!{
                day_26.clone() => vec![comment_content.clone()],
                day_27.clone() => vec![comment_content.clone()]
            }),
            get_unique_int_str("comments.day_to_comments.{}.title") =>to_value(hashmap!{
                day_26.clone() => comment_title_26.clone(),
                day_27.clone() => comment_title_27.clone()
            }),
            get_unique_int_str("comments.day_to_comments.{}.content") =>to_value(hashmap!{
                day_26.clone() => comment_content_26.clone(),
                day_27.clone() => comment_content_27.clone()
            })
        });

    assert_eq!(encode(&decoded, &Blog::get_field_int_map()), encoded);
    assert_eq!(decode(&encoded, &revert(Blog::get_field_int_map())),
               decoded);
}
