#![feature(plugin, test)]
#![plugin(arthas_plugin)]

extern crate test;
extern crate rand;
extern crate arthas;
extern crate model;
extern crate env_logger;

pub mod common;

use std::time::Duration;
use std::thread::sleep;
use model::*;
use common::memory_setup;


#[test]
#[ignore]
fn test_memory_leaks() {
    memory_setup();

    for _ in 0..10000 {
        let id = Article::session()
            .insert(Article::new("Hello world!"))
            .unwrap();

        Article::session().id(&id).remove().unwrap();
    }

    sleep(Duration::from_secs(3));
}
