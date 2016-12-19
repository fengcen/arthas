
extern crate arthas;
extern crate mktemp;
extern crate rand;
extern crate env_logger;

use self::mktemp::Temp;
use std::sync::{Once, ONCE_INIT};
use rand::random;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;


pub fn setup() {
    static ONCE: Once = ONCE_INIT;

    ONCE.call_once(|| {
        config_env_logger();
        let temp_dir = Temp::new_dir().unwrap().to_path_buf();
        arthas::config::path(temp_dir);
    });
}


pub fn memory_setup() {
    static ONCE: Once = ONCE_INIT;

    ONCE.call_once(|| {
        config_env_logger();
        arthas::config::persistence(false);
    });
}

pub fn random_usize() -> usize {
    random::<usize>()
}

pub fn random_string() -> String {
    let len = random_usize_max(15);
    rand::thread_rng()
        .gen_iter::<char>()
        .take(len)
        .collect()
}

fn random_usize_max(max: usize) -> usize {
    let between = Range::new(0, max);
    let mut rng = rand::thread_rng();
    between.ind_sample(&mut rng)
}

fn config_env_logger() {
    env_logger::init().unwrap();
}
