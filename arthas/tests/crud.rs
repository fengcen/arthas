#![feature(plugin, custom_derive)]
#![plugin(arthas_plugin)]

extern crate rand;
extern crate arthas;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;

pub mod common;
pub mod model;

use model::*;
use common::setup;


#[test]
fn test_a_insert() {
    setup();

    let id = Article::session().insert(Article::new("Hello world!")).unwrap();
    let item = Article::session().id(&id).find_one().unwrap();
    assert!(item.is_some());
    assert_eq!(item.unwrap().title, "Hello world!");
}

#[test]
fn test_a_insert_random() {
    setup();

    Article::session()
        .insert(Article::new("Hello world!").views(common::random_usize()))
        .unwrap();
}

#[test]
fn test_remove() {
    setup();

    let id = Article::session().insert(Article::new("Hello world!")).unwrap();
    let items = Article::session().id(&id).remove().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items.first().unwrap().title, "Hello world!");
}

#[test]
fn test_update() {
    setup();

    let id = Article::session().insert(Article::new("Hello world!")).unwrap();
    Article::session().id(&id).update(|item| item.title = "Hello".to_owned()).unwrap();
    let item = Article::session().id(&id).find_one().unwrap();
    assert!(item.is_some());
    assert_eq!(item.unwrap().title, "Hello");
}

#[test]
fn test_replace() {
    setup();

    let id = Article::session().insert(Article::new("Hello world!")).unwrap();
    Article::session().id(&id).replace(Article::new("Foobar Replace!")).unwrap();
    let item = Article::session().id(&id).find_one().unwrap();
    assert_eq!(item.unwrap().title, "Foobar Replace!");
}

#[test]
fn test_find() {
    setup();

    for i in 0..10 {
        Article::session()
            .insert(Article::new("Foobar!").views(i))
            .unwrap();
    }

    let all_items = Article::session()
        .field("title")
        .eq("Foobar!")
        .find()
        .unwrap();

    assert_eq!(all_items.len(), 10);

    let less_five_items = Article::session()
        .field("title")
        .eq("Foobar!")
        .field("views")
        .lt(5)
        .find()
        .unwrap();

    assert_eq!(less_five_items.len(), 5);
}

#[test]
fn test_count() {
    setup();

    for i in 0..10 {
        Article::session()
            .insert(Article::new("Count!").views(i))
            .unwrap();
    }

    let count = Article::session()
        .field("title")
        .eq("Count!")
        .count()
        .unwrap();

    assert_eq!(count, 10);
}

#[test]
fn test_offset() {
    setup();

    for i in 0..10 {
        Article::session()
            .insert(Article::new("Offset!").views(i))
            .unwrap();
    }

    let count = Article::session()
        .field("title")
        .eq("Offset!")
        .offset(5)
        .count()
        .unwrap();

    assert_eq!(count, 5);
}

#[test]
fn test_lt_100() {
    setup();

    Article::session()
        .field("views")
        .lt(100)
        .find()
        .unwrap();
}

#[test]
fn test_gt_100() {
    setup();

    Article::session()
        .field("views")
        .gt(100)
        .find()
        .unwrap();
}

#[test]
fn test_desc() {
    setup();

    for i in 0..5 {
        Article::session()
            .insert(Article::new("Desc!").views(i))
            .unwrap();
    }

    let items = Article::session()
        .field("title")
        .eq("Desc!")
        .field("views")
        .lt(3)
        .desc("views")
        .find()
        .unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].views, 2);
    assert_eq!(items[1].views, 1);
    assert_eq!(items[2].views, 0);
}

#[test]
fn test_asc() {
    setup();

    for i in 0..5 {
        Article::session()
            .insert(Article::new("Asc!").views(i))
            .unwrap();
    }

    let items = Article::session()
        .field("title")
        .eq("Asc!")
        .field("views")
        .lt(3)
        .asc("views")
        .find()
        .unwrap();

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].views, 0);
    assert_eq!(items[1].views, 1);
    assert_eq!(items[2].views, 2);
}

#[test]
fn test_multiple_order() {
    setup();

    for i in 0..5 {
        let content = if i < 3 { "a" } else { "b" };
        Article::session()
            .insert(Article::new("Multiple Order!").views(i).content(content))
            .unwrap();
    }

    let items = Article::session()
        .field("title")
        .eq("Multiple Order!")
        .desc("content")
        .asc("views")
        .find()
        .unwrap();

    assert_eq!(items.len(), 5);
    assert_eq!(items[0].content, "b");
    assert_eq!(items[1].content, "b");
    assert_eq!(items[2].content, "a");
    assert_eq!(items[3].content, "a");
    assert_eq!(items[4].content, "a");

    assert_eq!(items[0].views, 3);
    assert_eq!(items[1].views, 4);
    assert_eq!(items[2].views, 0);
    assert_eq!(items[3].views, 1);
    assert_eq!(items[4].views, 2);

    let items = Article::session()
        .field("title")
        .eq("Multiple Order!")
        .desc("content")
        .desc("views")
        .find()
        .unwrap();

    assert_eq!(items[0].views, 4);
    assert_eq!(items[1].views, 3);
    assert_eq!(items[2].views, 2);
    assert_eq!(items[3].views, 1);
    assert_eq!(items[4].views, 0);
}

#[test]
fn test_len() {
    setup();

    for i in 0..5 {
        Article::session()
            .insert(Article::new("qwertyuiopasdfghjklzxcvbnm").views(i))
            .unwrap();
    }

    let items = Article::session()
        .len("title")
        .eq(26)
        .find()
        .unwrap();

    assert_eq!(items.len(), 5);
}

#[test]
fn test_empty_query_all() {
    setup();

    Article::session().insert(Article::new("Empty Query All!")).unwrap();

    let items = Article::session()
        .limit(100)
        .find()
        .unwrap();

    assert!(items.len() > 0);
}
