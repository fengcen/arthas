
use std::collections::BTreeMap;
use serde_json::Value;
use traits::FieldIntMap;
use std::iter::IntoIterator;
use bincode::SizeLimit;
use bincode::serde::{deserialize, serialize};


pub fn encode_wrapper(value: &Value, field_int_map: &FieldIntMap) -> Value {
    handle_wrapper(Action::Encode, value, field_int_map)
}

pub fn decode_wrapper(value: &Value, field_int_map: &FieldIntMap) -> Value {
    handle_wrapper(Action::Decode, value, field_int_map)
}

enum Action {
    Encode,
    Decode,
}

fn handle_wrapper(action: Action, value: &Value, field_int_map: &FieldIntMap) -> Value {
    if value.is_object() {
        let handled = match action {
            Action::Encode => encode(value.find("item").unwrap(), field_int_map),
            Action::Decode => decode(value.find("item").unwrap(), field_int_map),
        };

        let mut map = BTreeMap::new();
        map.insert("id".to_owned(), value.find("id").unwrap().to_owned());
        map.insert("item".to_owned(), handled);
        Value::Object(map)
    } else {
        unreachable!()
    }
}

pub fn encode(value: &Value, field_int_map: &FieldIntMap) -> Value {
    let mut output = Value::Object(BTreeMap::new());

    for (path, integer) in field_int_map {
        encode_segments(path,
                        &mut get_path_segments(path),
                        &mut Vec::new(),
                        value,
                        object_entry(&mut output, integer));
    }

    output
}

fn encode_segments<'a>(path: &str,
                       segments: &mut Vec<&'a str>,
                       current_path: &mut Vec<&'a str>,
                       current_input: &Value,
                       current_output: &mut Value) {
    let segment = segments.remove(0);
    current_path.push(segment);

    if segment == "[]" {
        if current_input.as_array().is_some() {
            for (index, input) in current_input.as_array().unwrap().into_iter().enumerate() {
                encode_segments(path,
                                segments,
                                current_path,
                                input,
                                array_get_value_mut(current_output, index));
            }
        }
    } else if segment == "{}" {
        if current_input.as_object().is_some() {
            for (field, input) in current_input.as_object().unwrap() {
                encode_segments(path,
                                segments,
                                current_path,
                                input,
                                object_entry_value_mut(current_output, field));
            }
        }
    } else if current_path.join(".") == path {
        *current_output = current_input.find(segment).unwrap().clone();
    } else {
        encode_segments(path,
                        segments,
                        current_path,
                        current_input.find(segment).unwrap(),
                        current_output);
    }

    segments.insert(0, segment);
    current_path.pop();
}

pub fn decode(value: &Value, int_field_map: &FieldIntMap) -> Value {
    let mut output = Value::Object(BTreeMap::new());
    let path_value = replace_integer(value, int_field_map);

    for (path, input) in path_value.as_object().unwrap() {
        decode_segments(path,
                        &mut get_path_segments(path),
                        &mut Vec::new(),
                        input,
                        &mut output);
    }

    output
}

fn decode_segments<'a>(path: &str,
                       segments: &mut Vec<&'a str>,
                       current_path: &mut Vec<&'a str>,
                       current_input: &Value,
                       current_output: &mut Value) {
    let segment = segments.remove(0);
    current_path.push(segment);

    if segment == "[]" {
        if current_input.as_array().is_some() {
            for (index, input) in current_input.as_array().unwrap().into_iter().enumerate() {
                decode_segments(path,
                                segments,
                                current_path,
                                input,
                                array_get_value_mut(current_output, index));
            }
        } else {
            *current_output = Value::Array(Vec::new());
        }
    } else if segment == "{}" {
        if current_input.as_object().is_some() {
            for (field, input) in current_input.as_object().unwrap() {
                decode_segments(path,
                                segments,
                                current_path,
                                input,
                                object_entry_value_mut(current_output, field));
            }
        } else {
            *current_output = Value::Object(BTreeMap::new());
        }
    } else if current_path.join(".") == path {
        object_insert(current_output, segment, current_input);
    } else {
        decode_segments(path,
                        segments,
                        current_path,
                        current_input,
                        object_entry_value_mut(current_output, segment));
    }

    segments.insert(0, segment);
    current_path.pop();
}

#[inline]
pub fn bin_encode<T: Into<String>>(input: T) -> Vec<u8> {
    serialize(&input.into(), SizeLimit::Infinite).unwrap()
}

#[inline]
pub fn bin_decode<T: AsRef<[u8]>>(input: T) -> String {
    deserialize(input.as_ref()).unwrap()
}

#[inline]
fn replace_integer(value: &Value, int_field_map: &FieldIntMap) -> Value {
    let mut map = BTreeMap::new();

    for (integer, value) in value.as_object().unwrap() {
        map.insert(int_field_map.get(integer).cloned().unwrap(), value.clone());
    }

    Value::Object(map)

}

#[inline]
fn get_path_segments(path: &str) -> Vec<&str> {
    path.split('.').collect::<Vec<_>>()
}

#[inline]
fn object_entry<'a>(object: &'a mut Value, field: &str) -> &'a mut Value {
    if let Value::Object(ref mut map) = *object {
        map.entry(field.to_owned()).or_insert(Value::Null)
    } else {
        unreachable!()
    }
}

#[inline]
fn object_insert(object: &mut Value, field: &str, input: &Value) {
    if let Value::Object(ref mut map) = *object {
        map.insert(field.to_owned(), input.clone());
    } else {
        unreachable!()
    }
}

#[inline]
fn array_get_value_mut(array: &mut Value, index: usize) -> &mut Value {
    if array.as_array().is_none() {
        *array = Value::Array(Vec::new());
    }

    if let Value::Array(ref mut vec) = *array {
        if vec.get(index).is_none() {
            vec.push(Value::Object(BTreeMap::new()));
        }

        &mut vec[index]
    } else {
        unreachable!()
    }
}

#[inline]
fn object_entry_value_mut<'a>(object: &'a mut Value, field: &str) -> &'a mut Value {
    if object.as_object().is_none() {
        *object = Value::Object(BTreeMap::new());
    }

    if let Value::Object(ref mut map) = *object {
        map.entry(field.to_owned()).or_insert_with(|| Value::Object(BTreeMap::new()))
    } else {
        unreachable!()
    }
}
