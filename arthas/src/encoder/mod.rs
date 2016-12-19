
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


#[cfg(test)]
mod tests {
    use test;
    use std::collections::HashMap;
    use serde_json::to_value;
    use super::{encode, decode};
    use traits::get_unique_int_str;
    use utils::hash_map::revert;
    use model::*;


    #[bench]
    fn bench_encode(b: &mut test::Bencher) {
        let title = "This is title".to_owned();
        let content = "This is content".to_owned();
        let comment_title = "This is comment title".to_owned();
        let comment_content = "This is comment content".to_owned();
        let field_int_map = Article::get_field_int_map();

        let value = to_value(Article {
            _id: String::new(),
            title: title.clone(),
            content: content.clone(),
            day_to_views: HashMap::new(),
            views: 0,
            comments: vec![Comment {
                               title: comment_title.clone(),
                               content: comment_content.clone(),
                           }],
        });

        b.iter(|| encode(&value, &field_int_map))
    }

    #[bench]
    fn bench_decode(b: &mut test::Bencher) {
        let title = "This is title".to_owned();
        let content = "This is content".to_owned();
        let comment_title = "This is comment title".to_owned();
        let comment_content = "This is comment content".to_owned();
        let int_field_map = revert(Article::get_field_int_map());

        let value = to_value(hashmap!{
            get_unique_int_str("_id") => to_value(""),
            get_unique_int_str("title") => to_value(title.clone()),
            get_unique_int_str("content") => to_value(content.clone()),
            get_unique_int_str("day_to_views") => to_value(HashMap::<String, usize>::new()),
            get_unique_int_str("views") => to_value(0),
            get_unique_int_str("comments.[].title") => to_value(vec![comment_title.clone()]),
            get_unique_int_str("comments.[].content") => to_value(vec![comment_content.clone()])
        });

        b.iter(|| decode(&value, &int_field_map))
    }

    #[test]
    fn test_atomic() {
        let string_one = "This is string one".to_owned();
        let string_two = "This is string two".to_owned();

        let decoded = to_value(Atomic {
            string_one: string_one.clone(),
            string_two: string_two.clone(),
            hash_map: HashMap::new(),
        });

        let encoded = to_value(hashmap!{
            get_unique_int_str("string_one") => to_value(string_one.clone()),
            get_unique_int_str("string_two") => to_value(string_two.clone()),
            get_unique_int_str("hash_map") => to_value(HashMap::<String, usize>::new())
        });

        assert_eq!(encode(&decoded, &Atomic::get_field_int_map()), encoded);
        assert_eq!(decode(&encoded, &revert(Atomic::get_field_int_map())),
                   decoded);
    }

    #[test]
    fn test_vec() {
        let title = "This is title".to_owned();
        let content = "This is content".to_owned();
        let comment_title = "This is comment title".to_owned();
        let comment_content = "This is comment content".to_owned();

        let decoded = to_value(Article {
            _id: String::new(),
            title: title.clone(),
            content: content.clone(),
            day_to_views: HashMap::new(),
            views: 0,
            comments: vec![Comment {
                               title: comment_title.clone(),
                               content: comment_content.clone(),
                           }],
        });

        let encoded = to_value(hashmap!{
            get_unique_int_str("_id") => to_value(""),
            get_unique_int_str("title") => to_value(title.clone()),
            get_unique_int_str("content") => to_value(content.clone()),
            get_unique_int_str("day_to_views") => to_value(HashMap::<String, usize>::new()),
            get_unique_int_str("views") => to_value(0),
            get_unique_int_str("comments.[].title") => to_value(vec![comment_title.clone()]),
            get_unique_int_str("comments.[].content") => to_value(vec![comment_content.clone()])
        });

        assert_eq!(encode(&decoded, &Article::get_field_int_map()), encoded);
        assert_eq!(decode(&encoded, &revert(Article::get_field_int_map())),
                   decoded);
    }

    #[test]
    fn test_hashmap() {
        let day_26 = "2016-10-26".to_owned();
        let day_27 = "2016-10-27".to_owned();
        let title_26 = "This is day 26 title".to_owned();
        let title_27 = "This is day 27 title".to_owned();
        let content_26 = "This is day 26 content".to_owned();
        let content_27 = "This is day 27 content".to_owned();

        let decoded = to_value(Comments {
            day_to_comments: hashmap!{
                day_26.clone() => Comment {
                    title: title_26.clone(),
                    content: content_26.clone()
                },
                day_27.clone() => Comment {
                    title: title_27.clone(),
                    content: content_27.clone()
                }
            },
        });

        let encoded = to_value(hashmap!{
            get_unique_int_str("day_to_comments.{}.title") => to_value(hashmap!{
                day_26.clone() => title_26.clone(),
                day_27.clone() => title_27.clone()
            }),
            get_unique_int_str("day_to_comments.{}.content") => to_value(hashmap!{
                day_26.clone() => content_26.clone(),
                day_27.clone() => content_27.clone()
            })
        });

        assert_eq!(encode(&decoded, &Comments::get_field_int_map()), encoded);
        assert_eq!(decode(&encoded, &revert(Comments::get_field_int_map())),
                   decoded);
    }

    #[test]
    fn test_blog() {
        let comment_title = "This is comment title".to_owned();
        let comment_content = "This is comment content".to_owned();
        let day_26 = "2016-10-26".to_owned();
        let day_27 = "2016-10-27".to_owned();
        let title_26 = "This is day 26 title".to_owned();
        let title_27 = "This is day 27 title".to_owned();
        let content_26 = "This is day 26 content".to_owned();
        let content_27 = "This is day 27 content".to_owned();
        let comment_title_26 = "This is day 26 comment title".to_owned();
        let comment_title_27 = "This is day 27 comment title".to_owned();
        let comment_content_26 = "This is day 26 comment content".to_owned();
        let comment_content_27 = "This is day 27 comment content".to_owned();

        let decoded = to_value(Blog {
            articles: Articles {
                day_to_articles: hashmap!{
                    day_26.clone() => Article {
                        _id: String::new(),
                        title: title_26.clone(),
                        content: content_26.clone(),
                        day_to_views: HashMap::new(),
                        views: 0,
                        comments: vec![Comment {
                            title: comment_title.clone(),
                            content: comment_content.clone()
                        }]
                    },
                    day_27.clone() => Article {
                        _id: String::new(),
                        title: title_27.clone(),
                        content: content_27.clone(),
                        day_to_views: HashMap::new(),
                        views: 0,
                        comments: vec![Comment {
                            title: comment_title.clone(),
                            content: comment_content.clone()
                        }]
                    }
                },
            },
            comments: Comments {
                day_to_comments: hashmap!{
                    day_26.clone() => Comment {
                        title: comment_title_26.clone(),
                        content: comment_content_26.clone()
                    },
                    day_27.clone() => Comment {
                        title: comment_title_27.clone(),
                        content: comment_content_27.clone()
                    }
                },
            },
        });

        let encoded = to_value(hashmap!{
            get_unique_int_str("articles.day_to_articles.{}._id") => to_value(hashmap!{
                day_26.clone() => String::new(),
                day_27.clone() => String::new()
            }),
            get_unique_int_str("articles.day_to_articles.{}.title") => to_value(hashmap!{
                day_26.clone() => title_26.clone(),
                day_27.clone() => title_27.clone()
            }),
            get_unique_int_str("articles.day_to_articles.{}.content") => to_value(hashmap!{
                day_26.clone() => content_26.clone(),
                day_27.clone() => content_27.clone()
            }),
            get_unique_int_str("articles.day_to_articles.{}.day_to_views") => to_value(hashmap!{
                day_26.clone() => HashMap::<String, usize>::new(),
                day_27.clone() => HashMap::<String, usize>::new()
            }),
            get_unique_int_str("articles.day_to_articles.{}.views") => to_value(hashmap!{
                day_26.clone() => 0,
                day_27.clone() => 0
            }),
            get_unique_int_str("articles.day_to_articles.{}.comments.[].title") => to_value(hashmap!{
                day_26.clone() => vec![comment_title.clone()],
                day_27.clone() => vec![comment_title.clone()]
            }),
            get_unique_int_str("articles.day_to_articles.{}.comments.[].content") => to_value(hashmap!{
                day_26.clone() => vec![comment_content.clone()],
                day_27.clone() => vec![comment_content.clone()]
            }),
            get_unique_int_str("comments.day_to_comments.{}.title") =>to_value(hashmap!{
                day_26.clone() => comment_title_26.clone(),
                day_27.clone() => comment_title_27.clone()
            }),
            get_unique_int_str("comments.day_to_comments.{}.content") =>to_value(hashmap!{
                day_26.clone() => comment_content_26.clone(),
                day_27.clone() => comment_content_27.clone()
            })
        });

        assert_eq!(encode(&decoded, &Blog::get_field_int_map()), encoded);
        assert_eq!(decode(&encoded, &revert(Blog::get_field_int_map())),
                   decoded);
    }
}
