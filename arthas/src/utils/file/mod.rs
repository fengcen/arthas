
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all, OpenOptions};
use store::{get_path, get_extension_path};
use {DATA_EXTENSION, PERSISTENCE_EXTENSION, LOG_EXTENSION, SAVING_EXTENSION};


pub fn exists<P: AsRef<Path>>(file_path: P) -> bool {
    file_path.as_ref().is_file()
}

pub fn open_log_file(struct_name: &str) -> File {
    ensure_data_file_exists(struct_name, LOG_EXTENSION);
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(get_extension_path(struct_name, LOG_EXTENSION))
        .unwrap()
}

pub fn open_index_with_read(struct_name: &str) -> Option<File> {
    let path = get_extension_path(struct_name, PERSISTENCE_EXTENSION);
    if !path.is_file() {
        return None;
    }

    Some(OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap())
}

pub fn open_data_with_read(struct_name: &str) -> File {
    OpenOptions::new()
        .read(true)
        .open(get_extension_path(struct_name, DATA_EXTENSION))
        .unwrap()
}

pub fn open_data_or_create(struct_name: &str) -> File {
    ensure_data_file_exists(struct_name, DATA_EXTENSION);
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(get_extension_path(struct_name, DATA_EXTENSION))
        .unwrap()
}

pub fn open_data_with_append(struct_name: &str) -> File {
    ensure_data_file_exists(struct_name, DATA_EXTENSION);
    OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(get_extension_path(struct_name, DATA_EXTENSION))
        .unwrap()
}

pub fn open_with_write<P: AsRef<Path>>(path: P) -> File {
    ensure_file_exists(path.as_ref());
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap()
}

pub fn open_with_read<P: AsRef<Path>>(path: P) -> File {
    OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap()
}

pub fn get_log_path(struct_name: &str) -> PathBuf {
    get_extension_path(struct_name, LOG_EXTENSION)
}

pub fn get_persistence_path(struct_name: &str) -> PathBuf {
    get_extension_path(struct_name, PERSISTENCE_EXTENSION)
}

pub fn get_data_path(struct_name: &str) -> PathBuf {
    get_extension_path(struct_name, DATA_EXTENSION)
}

pub fn get_saving_path(struct_name: &str) -> PathBuf {
    get_extension_path(struct_name, SAVING_EXTENSION)
}

fn ensure_data_file_exists(struct_name: &str, ext: &'static str) {
    create_data_dir();
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(get_extension_path(struct_name, ext))
        .unwrap();
}

fn ensure_file_exists<P: AsRef<Path>>(path: P) {
    create_data_dir();
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
        .unwrap();
}

fn create_data_dir() {
    create_dir_all(get_path()).unwrap();
}
