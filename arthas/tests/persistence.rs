#![feature(plugin)]
#![plugin(arthas_plugin)]

extern crate rand;
extern crate arthas;
extern crate model;
extern crate env_logger;

pub mod common;

use model::*;
use common::setup;


#[test]
fn test_persistence_insert() {
    setup();

    for i in 0..10 {
        Article::session()
            .insert(Article::new("Foobar!").views(i))
            .unwrap();
    }
}
