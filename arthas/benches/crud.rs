#![cfg_attr(all(feature = "unstable", test), feature(test))]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate arthas_derive;
extern crate rand;
extern crate arthas;
extern crate env_logger;

#[path = "../tests/common/mod.rs"]
pub mod common;
#[path = "../tests/model/mod.rs"]
pub mod model;

#[cfg(all(feature = "unstable", test))]
mod benches {
    extern crate test;

    use model::*;
    use super::common::setup;

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
}
