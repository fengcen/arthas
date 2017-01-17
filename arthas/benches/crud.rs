#![feature(plugin, custom_derive, test)]
#![plugin(arthas_plugin)]

#[macro_use]
extern crate serde_derive;
extern crate test;
extern crate rand;
extern crate arthas;
extern crate env_logger;

#[path = "../tests/common/mod.rs"]
pub mod common;
#[path = "../tests/model/mod.rs"]
pub mod model;

use model::*;
use common::setup;


#[bench]
fn bench_a_insert(b: &mut test::Bencher) {
    setup();

    b.iter(|| {
        Article::session()
            .insert(Article::new("Hello world!"))
            .unwrap()
    })
}

#[bench]
fn bench_find(b: &mut test::Bencher) {
    setup();

    b.iter(|| {
        Article::session()
            .field("title")
            .eq("Hello world!")
            .limit(100)
            .find()
            .unwrap()
    })
}
