#![feature(plugin, custom_derive, proc_macro)]
#![plugin(arthas_plugin)]

#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate arthas;
extern crate env_logger;

pub mod common;
pub mod model;

use model::*;
use std::thread::spawn;
use common::setup;


#[test]
fn test_concurrent() {
    setup();

    let mut threads = Vec::new();

    for _ in 0..10 {
        threads.push(spawn(|| {
            for _ in 0..5 {
                insert();
                remove();
                find();
                replace();
            }
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }
}

fn insert() {
    let id = Article::session().insert(Article::new("Hello world!")).unwrap();
    let item = Article::session().id(&id).find_one().unwrap();
    assert!(item.is_some());
    assert_eq!(item.unwrap().title, "Hello world!");
}

fn remove() {
    let id = Article::session().insert(Article::new("Hello world!")).unwrap();
    let items = Article::session().id(&id).remove().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items.first().unwrap().title, "Hello world!");
}

fn replace() {
    let id = Article::session().insert(Article::new("Hello world!")).unwrap();
    Article::session().id(&id).replace(Article::new("Foobar Replace!")).unwrap();
    let item = Article::session().id(&id).find_one().unwrap();
    assert_eq!(item.unwrap().title, "Foobar Replace!");
}

fn find() {
    for i in 0..10 {
        Article::session()
            .insert(Article::new("Foobar!").views(i))
            .unwrap();
    }

    Article::session()
        .field("title")
        .eq("Foobar!")
        .find()
        .unwrap();
}
