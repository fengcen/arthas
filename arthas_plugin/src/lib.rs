//! A compiler plugin for [Arthas](https://docs.rs/arthas)
//!
#![feature(quote, plugin_registrar, rustc_private)]
extern crate syntax;
extern crate ecp;
extern crate regex;

use ecp::prelude::*;
use ecp::utils;
use regex::Regex;


#[plugin_registrar]
pub fn register(reg: &mut Registry) {
    reg.register_syntax_extension(utils::to_name("arthas"),
                                  SyntaxExtension::MultiModifier(Box::new(arthas)));
    reg.register_syntax_extension(utils::to_name("arthas_rename"),
                                  SyntaxExtension::MultiModifier(Box::new(arthas_rename)));
}

/// Attribute `#[arthas_rename = ""]`.
pub fn arthas_rename(cx: &mut ExtCtxt,
                     sp: Span,
                     meta: &MetaItem,
                     annotatable: Annotatable)
                     -> Vec<Annotatable> {
    let annotatable_builder = AnnotatableBuilder::from(&annotatable);
    let struct_builder = StructBuilder::from(cx, &annotatable_builder.get_item());
    let meta_builder = MetaItemBuilder::from(meta);
    let value = meta_builder.get_name_value();

    if value.is_none() {
        cx.span_err(sp, "expect rename value");
    } else {
        let mut error = None;
        let mut rename_block_builder = BlockBuilder::new();
        rename_block_builder.stmt(quote_stmt!(cx, let mut rename_map = ::std::collections::HashMap::new();).unwrap());

        for segment in value.unwrap().split(',') {
            let mut key_option = None;
            let mut value_option = None;
            for seg in segment.trim().split('=') {
                if key_option.is_none() {
                    key_option = Some(seg.trim().to_owned())
                } else if value_option.is_none() {
                    value_option = Some(seg.trim().to_owned())
                } else {
                    error = Some("invalid rename value");
                }
            }

            if key_option.is_some() && value_option.is_some() {
                let key = key_option.unwrap();
                let value = value_option.unwrap();
                rename_block_builder.stmt(utils::semi_to_stmt(quote_expr!(cx, rename_map.insert($key.to_owned(), $value.to_owned());).unwrap()));
            } else {
                error = Some("invalid rename value");
            }
        }

        if error.is_some() {
            cx.span_err(sp, error.unwrap());
        } else {
            rename_block_builder.stmt(utils::expr_to_stmt(quote_expr!(cx, rename_map).unwrap()));
            let struct_name = struct_builder.get_name();
            let rename_map_block = rename_block_builder.build();
            let _impl = quote_item!(
                cx,
                impl ::arthas::traits::SchemaRename for $struct_name {
                    fn get_rename_map() -> ::arthas::traits::RenameMap {
                        $rename_map_block
                    }
                }
            )
                .unwrap()
                .unwrap();

            return vec![utils::to_annotatable(struct_builder.build()), utils::to_annotatable(_impl)];
        }
    }

    vec![utils::to_annotatable(struct_builder.build())]
}

/// Attribute `#[arthas]`.
pub fn arthas(cx: &mut ExtCtxt,
              _: Span,
              meta: &MetaItem,
              annotatable: Annotatable)
              -> Vec<Annotatable> {
    let annotatable_builder = AnnotatableBuilder::from(&annotatable);
    let meta_builder = MetaItemBuilder::from(meta);
    let words = meta_builder.get_words();
    let is_one = if !words.is_empty() {
        words.first().unwrap() == "one"
    } else {
        false
    };

    let mut struct_builder = StructBuilder::from(cx, &annotatable_builder.get_item());
    struct_builder.derive("Default");
    struct_builder.derive("Debug");
    struct_builder.derive("Clone");
    struct_builder.derive("PartialEq");
    struct_builder.derive("Serialize");
    struct_builder.derive("Deserialize");
    let struct_name = struct_builder.get_name();

    let mut has_id = false;
    let mut deep_struct_fields = Vec::new();
    let mut field_type_map_block = BlockBuilder::new();
    field_type_map_block.stmt(quote_stmt!(cx,let mut field_type_map = ::std::collections::HashMap::new()).unwrap());

    for mut field in struct_builder.get_fields() {
        let ident = field.ident.take().unwrap().to_string();

        if ident == "_id" {
            has_id = true;
        }

        let field_type = get_inner_type(format!("{:?}", field.ty));

        if is_vec(&field_type) && vec_is_struct_type(&field_type) {
            let generic_type = get_vec_generic_type(&field_type);
            field_type_map_block.stmt(utils::semi_to_stmt(quote_expr!(cx,
                field_type_map.insert($ident.to_owned(),
                    ::arthas::traits::FieldType::Array($generic_type::get_field_type_map()));)
                .unwrap()));

            let block = quote_block!(cx, {
                let mut vec = Vec::new();
                vec.push($generic_type::new_deep_empty());
                vec
            })
                .unwrap();
            deep_struct_fields.push(Field {
                ident: utils::to_spanned(utils::to_ident(ident)),
                span: DUMMY_SP,
                is_shorthand: false,
                expr: P(Expr {
                    id: DUMMY_NODE_ID,
                    span: DUMMY_SP,
                    attrs: ThinVec::new(),
                    node: ExprKind::Block(P(block)),
                }),
            });
        } else if is_hashmap(&field_type) && hashmap_is_struct_type(&field_type) {
            let generic_type = get_hashmap_generic_type(&field_type);
            field_type_map_block.stmt(utils::semi_to_stmt(quote_expr!(cx,
                field_type_map.insert($ident.to_owned(),
                    ::arthas::traits::FieldType::Object($generic_type::get_field_type_map()));)
                .unwrap()));

            let block = quote_block!(cx, {
                let mut map = ::std::collections::HashMap::new();
                map.insert(String::new(), $generic_type::new_deep_empty());
                map
            })
                .unwrap();
            deep_struct_fields.push(Field {
                ident: utils::to_spanned(utils::to_ident(ident)),
                span: DUMMY_SP,
                is_shorthand: false,
                expr: P(Expr {
                    id: DUMMY_NODE_ID,
                    span: DUMMY_SP,
                    attrs: ThinVec::new(),
                    node: ExprKind::Block(P(block)),
                }),
            });
        } else if is_option(&field_type) && option_is_struct_type(&field_type) {
            let generic_type = get_option_generic_type(&field_type);
            field_type_map_block.stmt(utils::semi_to_stmt(quote_expr!(cx,
                field_type_map.insert($ident.to_owned(),
                    ::arthas::traits::FieldType::Object($generic_type::get_field_type_map()));)
                .unwrap()));

            let block = quote_block!(cx, {
                Some($generic_type::new_deep_empty())
            })
                .unwrap();
            deep_struct_fields.push(Field {
                ident: utils::to_spanned(utils::to_ident(ident)),
                span: DUMMY_SP,
                is_shorthand: false,
                expr: P(Expr {
                    id: DUMMY_NODE_ID,
                    span: DUMMY_SP,
                    attrs: ThinVec::new(),
                    node: ExprKind::Block(P(block)),
                }),
            });
        } else if is_struct_type(&field_type) {
            let generic_type = utils::to_ident(field_type);
            field_type_map_block.stmt(utils::semi_to_stmt(quote_expr!(cx,
                field_type_map.insert($ident.to_owned(),
                    ::arthas::traits::FieldType::Struct($generic_type::get_field_type_map()));)
                .unwrap()));
        } else {
            field_type_map_block.stmt(utils::semi_to_stmt(quote_expr!(cx,
                field_type_map.insert($ident.to_owned(),
                    ::arthas::traits::FieldType::Atomic($field_type.to_owned()));)
                .unwrap()));
        }
    }

    field_type_map_block.stmt(utils::expr_to_stmt(quote_expr!(cx,
        field_type_map)
        .unwrap()));

    let mut deep_struct = quote_expr!(cx, $struct_name {
        ..Default::default()
    })
        .unwrap();
    if let ExprKind::Struct(_, ref mut struct_fields, _) = deep_struct.node {
        *struct_fields = deep_struct_fields;
    }

    let set_id_block = if has_id {
        quote_block!(cx, {self._id = id;}).unwrap()
    } else {
        quote_block!(cx, {}).unwrap()
    };

    let set_id_ident = if has_id {
        utils::to_ident("id")
    } else {
        utils::to_ident("_")
    };

    let get_id_expr = if has_id {
        quote_stmt!(cx, return self._id.clone();)
    } else {
        quote_stmt!(cx, return String::new();)
    };

    let is_one_ident = utils::to_ident(is_one.to_string());
    let has_id_ident = utils::to_ident(has_id.to_string());
    let deep_struct_p = P(deep_struct);
    let field_int_map_block = field_type_map_block.build();
    let _impl = quote_item!(
        cx,
        impl ::arthas::traits::Schema for $struct_name {
            fn session<'a>() -> ::arthas::Query<'a, Self> {
                ::arthas::Query::new()
            }

            fn is_one() -> bool {
                $is_one_ident
            }

            fn new_empty() -> Self {
                $struct_name {
                    ..Default::default()
                }
            }

            fn new_deep_empty() -> Self {
                $deep_struct_p
            }

            fn has_id() -> bool {
                $has_id_ident
            }

            fn set_id(&mut self, $set_id_ident: String) {
                $set_id_block
            }

            fn get_id(&self) -> String {
                $get_id_expr
            }

            fn get_field_type_map() -> ::arthas::traits::FieldTypeMap {
                $field_int_map_block
            }

            fn get_field_int_map() -> ::arthas::traits::FieldIntMap {
                let mut field_int_map = ::std::collections::HashMap::new();
                let field_type_map = Self::get_field_type_map();
                let mut paths = Vec::new();
                scan_inner_type(&mut field_int_map, &field_type_map, &mut paths);
                return field_int_map;

                fn scan_inner_type(field_int_map: &mut ::arthas::traits::FieldIntMap, field_type_map: &::arthas::traits::FieldTypeMap, paths: &mut Vec<String>) {
                    for (field, field_type) in field_type_map {
                        match *field_type {
                            ::arthas::traits::FieldType::Atomic(_) => {
                                paths.push(field.to_owned().to_owned());
                                let path = join(paths, ".");
                                field_int_map.insert(path.clone(), ::arthas::traits::get_unique_int_str(&path));
                                paths.pop();
                            },
                            ::arthas::traits::FieldType::Array(ref type_map) => {
                                paths.push(field.to_owned().to_owned());
                                paths.push("[]".to_owned());
                                scan_inner_type(field_int_map, type_map, paths);
                                paths.pop();
                                paths.pop();
                            },
                            ::arthas::traits::FieldType::Object(ref type_map) => {
                                paths.push(field.to_owned().to_owned());
                                paths.push("{}".to_owned());
                                scan_inner_type(field_int_map, type_map, paths);
                                paths.pop();
                                paths.pop();
                            },
                            ::arthas::traits::FieldType::Struct(ref type_map) => {
                                paths.push(field.to_owned().to_owned());
                                scan_inner_type(field_int_map, type_map, paths);
                                paths.pop();
                            }
                        }
                    }
                }

                fn join<T: AsRef<str>, I: AsRef<str>>(vec: &[T], separator: I) -> String {
                    let mut buffer = String::new();
                    for (count, item) in vec.iter().enumerate() {
                        if count == 0 {
                            buffer += item.as_ref();
                        } else {
                            buffer += separator.as_ref();
                            buffer += item.as_ref();
                        }
                    }

                    buffer
                }
            }
        }
    ).unwrap().unwrap();

    let mut has_rename_attr = false;
    for attr in struct_builder.get_attrs() {
        let attribute_builder = AttributeBuilder::from(attr);
        let name = attribute_builder.get_name();
        if name == "arthas_rename" {
            has_rename_attr = true;
            break;
        }
    }

    let mut items =
        vec![utils::to_annotatable(struct_builder.build()), utils::to_annotatable(_impl)];

    if !has_rename_attr {
        let impl_schema_rename =
            quote_item!(cx, impl ::arthas::traits::SchemaRename for $struct_name {
                fn get_rename_map() -> ::arthas::traits::RenameMap {
                    ::arthas::traits::RenameMap::new()
                }
            })
                .unwrap()
                .unwrap();
        items.push(utils::to_annotatable(impl_schema_rename));
    }

    items
}

fn get_inner_type(field_type: String) -> String {
    let reg = Regex::new(r"type\((.*)\)").unwrap();
    let cap = reg.captures(&field_type).unwrap();
    cap.at(1).unwrap().to_owned()
}

fn is_vec(type_: &str) -> bool {
    type_.starts_with("Vec<")
}

fn get_vec_generic_type(type_: &str) -> Ident {
    let reg = Regex::new(r"Vec<(.*)>").unwrap();
    let cap = reg.captures(type_).unwrap();
    utils::to_ident(cap.at(1).unwrap())
}

fn is_option(type_: &str) -> bool {
    type_.starts_with("Option<")
}

fn get_option_generic_type(type_: &str) -> Ident {
    let reg = Regex::new(r"Option<(.*)>").unwrap();
    let cap = reg.captures(type_).unwrap();
    utils::to_ident(cap.at(1).unwrap())
}

fn is_hashmap(type_: &str) -> bool {
    type_.starts_with("HashMap<")
}

fn get_hashmap_generic_type(type_: &str) -> Ident {
    let reg = Regex::new(r"HashMap<[a-z0-9A-Z]+, *(.*)>").unwrap();
    let cap = reg.captures(type_).unwrap();
    utils::to_ident(cap.at(1).unwrap())
}

fn hashmap_is_struct_type(type_: &str) -> bool {
    is_struct_type(&get_hashmap_generic_type(type_).to_string())
}

fn vec_is_struct_type(type_: &str) -> bool {
    is_struct_type(&get_vec_generic_type(type_).to_string())
}

fn option_is_struct_type(type_: &str) -> bool {
    is_struct_type(&get_option_generic_type(type_).to_string())
}

fn is_struct_type(generic_type: &str) -> bool {
    if is_option(generic_type) && !option_is_struct_type(generic_type) {
        false
    } else if is_vec(generic_type) && !vec_is_struct_type(generic_type) {
        false
    } else if is_hashmap(generic_type) && !hashmap_is_struct_type(generic_type) {
        false
    } else {
        match generic_type {
            "String" | "&'static str" | "usize" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" |
            "i32" | "i64" | "bool" => false,
            _ => true,
        }
    }
}
