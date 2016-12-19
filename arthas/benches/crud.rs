#![feature(plugin, test)]
#![plugin(arthas_plugin)]

extern crate test;
extern crate rand;
extern crate arthas;
extern crate model;
extern crate env_logger;

#[path = "../tests/common/mod.rs"]
pub mod common;

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
