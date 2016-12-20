#![feature(plugin, custom_derive, proc_macro, test)]
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
use common::memory_setup;


#[bench]
fn bench_a_insert_random(b: &mut test::Bencher) {
    memory_setup();

    b.iter(|| {
        Article::session()
            .insert(Article::new(common::random_string()).views(common::random_usize()))
            .unwrap()
    })
}

#[bench]
fn bench_b_insert_hello(b: &mut test::Bencher) {
    memory_setup();

    b.iter(|| {
        Article::session()
            .insert(Article::new("Hello world!"))
            .unwrap()
    })
}

#[bench]
fn bench_find(b: &mut test::Bencher) {
    memory_setup();

    b.iter(|| {
        Article::session()
            .field("title")
            .eq("Hello world!")
            .limit(100)
            .find()
            .unwrap()
    })
}

#[bench]
fn bench_lt_100(b: &mut test::Bencher) {
    memory_setup();

    b.iter(|| {
        Article::session()
            .field("views")
            .lt(100)
            .limit(100)
            .find()
            .unwrap()
    })
}

#[bench]
fn bench_gt_100(b: &mut test::Bencher) {
    memory_setup();

    b.iter(|| {
        Article::session()
            .field("views")
            .gt(100)
            .limit(100)
            .find()
            .unwrap()
    })
}

#[bench]
fn bench_multiple_conditions_find(b: &mut test::Bencher) {
    memory_setup();

    b.iter(|| {
        Article::session()
            .field("title")
            .eq("Hello world!")
            .field("views")
            .lt(100)
            .limit(100)
            .find()
            .unwrap()
    })
}

#[bench]
fn bench_count(b: &mut test::Bencher) {
    b.iter(|| {
        Article::session()
            .field("title")
            .eq("Hello world!")
            .count()
            .unwrap()
    })
}

#[bench]
fn bench_find_one(b: &mut test::Bencher) {
    b.iter(|| {
        Article::session()
            .field("title")
            .eq("Hello world!")
            .find_one()
            .unwrap()
    })
}

#[bench]
fn bench_limit_one(b: &mut test::Bencher) {
    b.iter(|| {
        Article::session()
            .field("title")
            .eq("Hello world!")
            .limit(1)
            .find()
            .unwrap()
    })
}
