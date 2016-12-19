
use std::path::PathBuf;
use std::env;
use DATA_DIR;


pub struct Config {
    pub persistence: bool,
    pub path: PathBuf,
}

impl Config {
    pub fn new() -> Config {
        Config {
            persistence: true,
            path: get_path(),
        }
    }
}

fn get_path() -> PathBuf {
    env::current_dir().unwrap().join(DATA_DIR)
}
