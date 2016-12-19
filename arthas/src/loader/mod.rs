
use std::io::prelude::*;
use std::str::from_utf8;
use std::str;
use std::sync::{RwLock, RwLockWriteGuard};
use std::io::BufReader;
use std::io::BufRead;
use std::fs::{self, File};
use memmap::{Mmap, Protection};
use serde_json::{self, Value, to_value};
use persistence::operation::Operation;
use traits::{FieldIntMap, RenameMap};
use encoder::{bin_encode, bin_decode};
use traits::get_unique_int_str;
use {utils, encoder};
use memory::Memory;
use traits::Structure;
use persistence::Persistence;
use config;
use utils::file::{get_log_path, get_data_path, get_persistence_path, get_saving_path};
use persistence::meta::SimpleMeta;
use BINENCODE;
use store::{is_persistence, persistences, memories, config};


pub fn load<T: Structure>() {
    let struct_name = utils::reflect::get_type_name::<T>();

    fix_persistence_name(&struct_name);
    check_log_and_persistence(&struct_name);

    if utils::file::exists(get_log_path(&struct_name)) {
        write_log(&struct_name);
    }

    let field_int_map = T::get_field_int_map();
    let rename_map = T::get_rename_map();
    let persistence_store_lock = persistences();
    let mut persistence_store = persistence_store_lock.write().unwrap();
    let mut persistence =
        persistence_store.entry(struct_name.clone())
            .or_insert_with(|| {
                RwLock::new(Persistence::new(struct_name.clone(), field_int_map.clone()))
            })
            .write()
            .unwrap();

    let memory_store_lock = memories();
    let mut memory_store = memory_store_lock.write().unwrap();
    let memory = memory_store.entry(struct_name.clone())
        .or_insert_with(|| RwLock::new(Memory::new::<T>()))
        .write()
        .unwrap();

    let persistence_option = load_persistence(&struct_name,
                                              &field_int_map,
                                              &rename_map,
                                              &serde_json::to_value(T::new_deep_empty()));

    if persistence_option.is_some() {
        *persistence = persistence_option.unwrap();
        load_data(memory, &persistence, &struct_name, &T::get_field_int_map());
    } else {
        create_persistence(&struct_name, &field_int_map);
    }

    config();
}

pub fn persistence_exits(struct_name: &str) -> bool {
    get_persistence_path(struct_name).is_file()
}

pub fn create_persistence(struct_name: &str, field_int_map: &FieldIntMap) {
    save_persistence(Persistence::new(struct_name.to_owned(), field_int_map.clone()));
    fix_persistence_name(struct_name);
}

pub fn load_persistence(struct_name: &str,
                        field_int_map: &FieldIntMap,
                        rename_map: &RenameMap,
                        empty_value: &Value)
                        -> Option<Persistence> {
    let persistence_path = get_persistence_path(struct_name);
    if !utils::file::exists(&persistence_path) {
        return None;
    }

    let persistence_option = read_persistence(struct_name);
    if persistence_option.is_none() {
        return None;
    }

    let mut persistence = persistence_option.unwrap();

    if to_value(&persistence.field_int_map).to_string() != to_value(field_int_map).to_string() {
        update_schema(&persistence,
                      struct_name,
                      field_int_map,
                      rename_map,
                      empty_value);
        persistence = read_persistence(struct_name).unwrap();
    }

    Some(persistence)


}

fn read_persistence(struct_name: &str) -> Option<Persistence> {
    let file = utils::file::open_index_with_read(struct_name);
    let mut buf = Vec::new();

    if file.is_some() {
        file.unwrap().read_to_end(&mut buf).unwrap();
    }

    if buf.is_empty() {
        None
    } else {
        match serde_json::from_str::<Persistence>(&if BINENCODE {
            bin_decode(&buf)
        } else {
            from_utf8(&buf).unwrap().to_owned()
        }) {
            Ok(persistence) => Some(persistence),
            Err(err) => panic!("Invalid persistence index: {:?}", err),
        }
    }
}

pub fn load_data(mut memory: RwLockWriteGuard<Memory>,
                 persistence: &Persistence,
                 struct_name: &str,
                 field_int_map: &FieldIntMap) {
    let path = get_data_path(struct_name);
    let enable_backup = is_persistence();
    config::persistence(false);

    if utils::file::exists(&path) {
        let file = utils::file::open_data_with_read(struct_name);
        if to_value(&persistence.field_int_map).to_string() != to_value(field_int_map).to_string() {
            panic!("Incompatible schema!");
        }

        for (id, meta) in &persistence.metas {
            let mmap = Mmap::open_with_offset(&file, Protection::Read, meta.offset(), meta.real())
                .unwrap();

            let bytes: &[u8] = unsafe { mmap.as_slice() };
            let value = serde_json::from_str::<Value>(&if BINENCODE {
                    bin_decode(bytes)
                } else {
                    from_utf8(bytes).unwrap().to_owned()
                })
                .unwrap();

            memory.insert_encoded_value(id.to_owned(), value).unwrap();
        }
    }

    config::persistence(enable_backup);
}


fn update_schema(persistence: &Persistence,
                 struct_name: &str,
                 field_int_map: &FieldIntMap,
                 rename_map: &RenameMap,
                 empty_value: &Value) {
    let old_data_path = get_data_path(struct_name);
    let old_data_index_path = get_persistence_path(struct_name);
    let backup_data_path = format!("{}.backup", old_data_path.display());
    let backup_data_index_path = format!("{}.backup", old_data_index_path.display());
    fs::rename(&old_data_path, &backup_data_path).unwrap();
    fs::rename(&old_data_index_path, &backup_data_index_path).unwrap();

    let old_file = utils::file::open_with_read(backup_data_path);
    let mut new_persistence = Persistence::new(struct_name.to_owned(), field_int_map.clone());

    for meta in persistence.metas.values() {
        let mmap = Mmap::open_with_offset(&old_file, Protection::Read, meta.offset(), meta.real())
            .unwrap();
        let bytes: &[u8] = unsafe { mmap.as_slice() };
        let mut value = serde_json::from_str::<Value>(&if BINENCODE {
                bin_decode(bytes)
            } else {
                from_utf8(bytes).unwrap().to_owned()
            })
            .unwrap();

        fix_value(&mut value, rename_map, empty_value, field_int_map);

        new_persistence.insert(Operation::new()
            .value(value)
            .insert());
    }
}

fn fix_value(value: &mut Value,
             rename_map: &RenameMap,
             empty_value: &Value,
             field_int_map: &FieldIntMap) {
    let integer_map = utils::hash_map::revert(field_int_map.clone());
    let object = value.as_object_mut().unwrap().get_mut("item").unwrap().as_object_mut().unwrap();
    let empty_value = encoder::encode(empty_value, field_int_map);
    let empty_object = empty_value.as_object().unwrap();
    let integers = object.keys().map(|v| v.to_owned()).collect::<Vec<_>>();
    let new_integers = integer_map.keys().collect::<Vec<_>>();

    for (from, to) in rename_map {
        let from_integer = get_unique_int_str(from);
        let to_integer = get_unique_int_str(to);
        let remove_value = object.remove(&from_integer);

        if remove_value.is_some() {
            object.insert(to_integer, remove_value.unwrap());
        }
    }

    for integer in integers {
        if !integer_map.contains_key(&integer) {
            object.remove(&integer);
        }
    }

    for integer in new_integers {
        if !object.contains_key(integer) {
            object.insert(integer.to_owned(),
                          empty_object.get(integer).cloned().unwrap());
        }
    }
}

fn fix_persistence_name(struct_name: &str) {
    let persistence_path = get_persistence_path(struct_name);
    let saving_path = get_saving_path(struct_name);
    if utils::file::exists(&persistence_path) && utils::file::exists(&saving_path) {
        fs::remove_file(saving_path).unwrap();
    } else if utils::file::exists(&saving_path) {
        fs::rename(saving_path, persistence_path).unwrap();
    }
}

fn check_log_and_persistence(struct_name: &str) {
    if utils::file::exists(get_log_path(struct_name)) &&
       !utils::file::exists(get_persistence_path(struct_name)) {
        panic!("log exists but index not found!");
    }
}

fn fix_file_length(struct_name: &str, size: usize) {
    utils::file::open_data_or_create(struct_name).set_len(size as u64).unwrap();
}

fn write_log(struct_name: &str) {
    let mut persistence = read_persistence(struct_name).unwrap();
    fix_file_length(struct_name, persistence.size);

    let log_file = utils::file::open_log_file(struct_name);
    let file = BufReader::new(&log_file);
    let mut append_file = utils::file::open_data_with_append(struct_name);
    let mmap_file = utils::file::open_data_or_create(struct_name);

    for line_result in file.lines() {
        let line = line_result.unwrap();
        let info = persistence.write_log(line);
        if info.is_some() {
            let (append, meta, bytes) = info.unwrap();
            save_bytes(&mut append_file, &mmap_file, append, meta, bytes);
        }
    }

    save_persistence(persistence);
    remove_old_persistence(struct_name);
    fix_persistence_name(struct_name);
    remove_log_file(struct_name);
}

fn remove_log_file(struct_name: &str) {
    fs::remove_file(get_log_path(struct_name)).unwrap();
}

fn save_persistence(persistence: Persistence) {
    utils::file::open_with_write(get_saving_path(&persistence.struct_name))
        .write_all(&if BINENCODE {
            bin_encode(to_value(persistence).to_string())
        } else {
            to_value(persistence).to_string().as_bytes().to_owned()
        })
        .unwrap();
}

fn remove_old_persistence(struct_name: &str) {
    fs::remove_file(get_persistence_path(struct_name)).unwrap();
}

fn save_bytes(append_file: &mut File,
              mmap_file: &File,
              append: bool,
              meta: SimpleMeta,
              data: Vec<u8>) {
    if append {
        append_file.write_all(&data).unwrap();
    } else {
        let mut mmap =
            Mmap::open_with_offset(mmap_file, Protection::ReadWrite, meta.offset(), meta.real())
                .unwrap();
        {
            let old_bytes: &mut [u8] = unsafe { mmap.as_mut_slice() };
            for (index, byte) in old_bytes.iter_mut().enumerate() {
                *byte = data[index];
            }
        }
    }
}
