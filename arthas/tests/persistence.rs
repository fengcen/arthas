
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate arthas_derive;
extern crate rand;
extern crate arthas;
extern crate env_logger;

pub mod common;
pub mod model;

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
