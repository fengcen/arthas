
mod config;

use std::sync::{RwLock, Once, ONCE_INIT};
use std::mem;
use std::collections::HashMap;
use std::path::PathBuf;
use self::config::Config;
use memory::Memory;
use persistence::Persistence;
use traits::Structure;


pub type MemoryStore = HashMap<String, RwLock<Memory>>;
pub type PersistenceStore = HashMap<String, RwLock<Persistence>>;

pub trait MemoryGetter {
    fn get_memory<T: Structure>(&self) -> &RwLock<Memory>;
}

impl MemoryGetter for MemoryStore {
    fn get_memory<T: Structure>(&self) -> &RwLock<Memory> {
        self.get(&T::get_struct_name()).unwrap()
    }
}


pub fn memories<'a>() -> &'a RwLock<MemoryStore> {
    static mut MEMORIES: *const RwLock<MemoryStore> = 0 as *const RwLock<MemoryStore>;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| MEMORIES = mem::transmute(Box::new(RwLock::new(MemoryStore::new()))));
        &*MEMORIES
    }
}

pub fn persistences<'a>() -> &'a RwLock<PersistenceStore> {
    static mut PERSISTENCES: *const RwLock<PersistenceStore> = 0 as *const RwLock<PersistenceStore>;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| {
            PERSISTENCES = mem::transmute(Box::new(RwLock::new(PersistenceStore::new())))
        });
        &*PERSISTENCES
    }
}

pub fn config<'a>() -> &'a RwLock<Config> {
    static mut CONFIG: *const RwLock<Config> = 0 as *const RwLock<Config>;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| CONFIG = mem::transmute(Box::new(RwLock::new(Config::new()))));
        &*CONFIG
    }
}

#[inline]
pub fn is_persistence() -> bool {
    config().read().unwrap().persistence
}

#[inline]
pub fn get_path() -> PathBuf {
    config().read().unwrap().path.clone()
}

#[inline]
pub fn get_extension_path(struct_name: &str, ext: &'static str) -> PathBuf {
    PathBuf::from(format!("{}/{}.{}", get_path().display(), struct_name, ext))
}
