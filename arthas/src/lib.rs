//! Arthas is an in-memory structure database.
//!
//! # Usage
//!
//! 1. Add dependencies to `Cargo.toml`.
//!
//!     ```html
//!     [dependencies]
//!     arthas = "^0.1"
//!     arthas_plugin = "^0.1"
//!     serde_derive = "^0.8"
//!     ```
//!
//! 2. In your `main.rs` or `lib.rs`:
//!
//!     ```html
//!     #![feature(plugin, custom_derive)]
//!     #![plugin(arthas_plugin)]
//!
//!     #[macro_use]
//!     extern crate serde_derive;
//!     extern crate arthas;
//!     ```
//!
//! 3. Add `[arthas]` attribute to your `struct`:
//!
//!     ```html
//!     #[arthas]
//!     pub struct Article {
//!         pub _id: String,    // If you want to use id. add a field named `_id`.
//!         pub title: String,
//!         pub content: String,
//!         pub views: usize,
//!     }
//!     ```
//!
//!
//! # CRUD Examples
//! All struct can use the static method `session()`. `session()` will return a [`Query`](struct.Query.html).
//!
//! ```
//! #![feature(plugin, custom_derive)]
//! #![plugin(arthas_plugin)]
//!
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate arthas;
//!
//! use arthas::prelude::*;
//!
//! #[arthas]
//! pub struct Article {
//!     pub _id: String,
//!     pub title: String,
//!     pub content: String,
//!     pub views: usize,
//! }
//!
//! impl Article {
//!     pub fn new<T: Into<String>>(title: T) -> Article {
//!         Article { title: title.into(), ..Default::default() }
//!     }
//! }
//!
//! fn main() {
//!     // Disable persistence for the tests.
//!     arthas::config::persistence(false);
//!
//!     // Insert
//!     let id = Article::session().insert(Article::new("Hello world!")).unwrap();
//!
//!     // Update
//!     Article::session().id(&id).update(|article| article.views = 10).unwrap();
//!
//!     // Find
//!     let items = Article::session().find().unwrap();
//!     assert!(items.len() > 0);
//!
//!     // Find One
//!     let item = Article::session().id(&id).find_one().unwrap();
//!     assert!(item.is_some());
//!     assert_eq!(item.unwrap().title, "Hello world!");
//!
//!     // Remove
//!     Article::session().id(&id).remove().unwrap();
//! }
//! ```
//!
//! # Load Persistence
//! Arthas will not automatically load persistence from disk, you have to load persistence yourself.
//!
//! ```html
//! arthas::load::<Article>();  // Load `Article`'s persistence.
//! ```
//!
//! # Update Structure
//! Sometimes you want to update your structure. Like renaming or removing fields. Arthas will automatically remove and add fields, but you have to tell Arthas if you want to **rename** fields.
//!
//! ```html
//! #[arthas]
//! #[arthas_rename = "content = body, views = visit"] // Use `[arthas_rename]` attribute.
//! pub struct Article {
//!     pub _id: String,
//!     pub title: String,
//!     pub body: String,   // This is the field renamed from `content`
//!     pub visit: usize,   // This is the field renamed from `views`
//! }
//! ```
//!
#![feature(core_intrinsics)]
#![cfg_attr(all(test), feature(test))]
#![deny(missing_docs)]

#[cfg(test)]
extern crate test;
#[cfg(test)]
extern crate env_logger;

#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate serde_json;
extern crate memmap;
extern crate bincode;
extern crate serde;
extern crate glob;
extern crate num_cpus;
extern crate scoped_pool;
extern crate quickersort;
extern crate objectid;
extern crate thread_id;
extern crate chrono;
extern crate vec_map;


#[macro_use]
mod macros;
#[doc(hidden)]
pub mod encoder;
mod memory;
mod persistence;
mod store;
mod utils;
mod loader;
mod query;
mod error;
mod tree;
mod item;

#[doc(hidden)]
pub mod traits;
pub mod prelude;

pub use query::Query;
pub use error::Error;
pub use item::Id;

#[doc(hidden)]
pub mod types;
pub mod config;


/// Load persistence.
pub fn load<T: traits::Structure>() {
    loader::load::<T>();
}


const DATA_EXTENSION: &'static str = "ar";
const PERSISTENCE_EXTENSION: &'static str = "arx";
const LOG_EXTENSION: &'static str = "arl";
const SAVING_EXTENSION: &'static str = "saving";
const DATA_DIR: &'static str = "arthas.ar";
const BINENCODE: bool = true;
