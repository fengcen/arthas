#![feature(plugin)]
#![plugin(arthas_plugin)]

extern crate rand;
extern crate arthas;
extern crate model;
extern crate env_logger;

pub mod common;

use model::*;
use arthas::Error;
use common::setup;

#[test]
fn test_can_not_replace() {
    setup();

    assert_eq!(Comment::session().replace(Comment { ..Default::default() }),
               Err(Error::CanNotReplace));
}

#[test]
fn test_field_not_found() {
    setup();

    assert_eq!(Article::session().field("bad field").eq("bad value").find(),
               Err(Error::FieldNotFound));
}
