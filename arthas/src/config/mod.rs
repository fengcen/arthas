//! Config Arthas.

use std::path::Path;
use store::config;


/// Set `false` to disable persistence. Defaults to `true`.
pub fn persistence(persistence: bool) {
    config().write().unwrap().persistence = persistence;
}

/// Set arthas root path. Defaults to `arthas.ar`.
pub fn path<P: AsRef<Path>>(path: P) {
    config().write().unwrap().path = path.as_ref().to_owned();
}
