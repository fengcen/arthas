
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate arthas_derive;
extern crate rand;
extern crate arthas;
extern crate env_logger;

pub mod common;
pub mod model;

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
