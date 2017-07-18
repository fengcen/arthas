//! Macros 1.1 implementation of #[derive(Arthas)]
//!
//! See [Arthas](https://github.com/fengcen/arthas) for more information.
//!
#![recursion_limit = "1000"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate regex;

use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::Ident;
use syn::Field;
use quote::ToTokens;
use regex::Regex;


#[proc_macro_derive(Arthas, attributes(arthas))]
#[doc(hidden)]
pub fn arthas(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_macro_input(&s).unwrap();
    let gen = impl_arthas(&ast);
    gen.parse().unwrap()
}

fn impl_arthas(ast: &DeriveInput) -> quote::Tokens {
    check_is_struct(ast);

    let name = &ast.ident;
    let name_string = name.to_string();
    let is_one = check_is_one(ast);
    let has_id = check_has_id(ast);

    let set_id_block = if has_id {
        quote! {self._id = id;}
    } else {
        quote! {{}}
    };

    let get_id_expr = if has_id {
        quote! {return self._id.clone();}
    } else {
        quote! {return String::new();}
    };

    let field_int_map_block = generate_field_int_map_block(ast);
    let deep_struct = generate_deep_struct(ast);
    let rename_map_block = generate_rename_map_block(ast);

    let set_id_ident = if has_id {
        quote! {id}
    } else {
        quote! {_}
    };

    quote! {
        impl ::arthas::traits::Arthas for #name {
            fn get_struct_name() -> String {
                #name_string.to_owned()
            }

            fn session<'a>() -> ::arthas::Query<'a, Self> {
                ::arthas::Query::new()
            }

            fn is_one() -> bool {
                #is_one
            }

            fn new_empty() -> Self {
                #name {
                    ..Default::default()
                }
            }

            fn new_deep_empty() -> Self {
                #deep_struct
            }

            fn has_id() -> bool {
                #has_id
            }

            fn set_id(&mut self, #set_id_ident: String) {
                #set_id_block
            }

            fn get_id(&self) -> String {
                #get_id_expr
            }

            fn get_field_type_map() -> ::arthas::traits::FieldTypeMap {
                #field_int_map_block
            }

            fn get_rename_map() -> ::std::collections::HashMap<String, String> {
                #rename_map_block
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
    }
}

fn get_ast_struct_fields(ast: &DeriveInput) -> &[Field] {
    if let syn::Body::Struct(ref variant_data) = ast.body {
        if let syn::VariantData::Struct(ref fields) = *variant_data {
            return fields;
        }
    }

    panic!("No fields in struct! Arthas does not support empty struct!");
}

fn generate_deep_struct(ast: &DeriveInput) -> quote::Tokens {
    let mut tokens = quote::Tokens::new();

    tokens.append(&format!("{} {{", ast.ident.to_string()));

    for field in get_ast_struct_fields(ast) {
        let field_ident = field.ident.clone().unwrap();
        let field_type = type_to_string(&field.ty);

        if is_vec(&field_type) && vec_is_struct_type(&field_type) {
            let generic_type = get_vec_generic_type(&field_type);
            tokens.append(&format!("{}:", field_ident.to_string()));
            tokens.append(&quote!{
                {
                    let mut vec = Vec::new();
                    vec.push(#generic_type::new_deep_empty());
                    vec
                }
            }
                .to_string());
            tokens.append(",");
        } else if is_hashmap(&field_type) && hashmap_is_struct_type(&field_type) {
            let generic_type = get_hashmap_generic_type(&field_type);
            tokens.append(&format!("{}:", field_ident.to_string()));
            tokens.append(&quote!{
                {
                    let mut map = ::std::collections::HashMap::new();
                    map.insert(String::new(), #generic_type::new_deep_empty());
                    map
                }
            }
                .to_string());
            tokens.append(",");
        } else if is_option(&field_type) && option_is_struct_type(&field_type) {
            let generic_type = get_option_generic_type(&field_type);
            tokens.append(&format!("{}:", field_ident.to_string()));
            tokens.append(&quote!{
                {
                    Some(#generic_type::new_deep_empty())
                }
            }
                .to_string());
            tokens.append(",");
        }
    }

    tokens.append("..Default::default()");
    tokens.append("}");
    tokens
}

fn get_rename_value(ast: &DeriveInput) -> String {
    for attr in &ast.attrs {
        if let syn::MetaItem::List(ref ident, ref meta_items) = attr.value {
            if ident.to_string() == "arthas" {
                for item in meta_items {
                    if let syn::NestedMetaItem::MetaItem(ref meta_item) = *item {
                        if let syn::MetaItem::NameValue(ref ident, ref lit) = *meta_item {
                            if ident.to_string() == "rename" {
                                if let syn::Lit::Str(ref value, _) = *lit {
                                    return value.to_owned();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    String::new()
}

fn generate_rename_map_block(ast: &DeriveInput) -> quote::Tokens {
    let rename_value = get_rename_value(ast);
    let mut rename_map_block = quote::Tokens::new();
    let mut rename_map_exists = false;
    rename_map_block.append("{let mut rename_map = ::std::collections::HashMap::new();");

    for name_value in rename_value.split(',') {
        if name_value.is_empty() {
            continue;
        }

        let segments = name_value.trim().split('=').collect::<Vec<_>>();
        if segments.len() != 2 {
            panic!("invalid rename value");
        }

        rename_map_exists = true;

        let name = segments[0];
        let value = segments[1];

        rename_map_block.append(&quote!{
            rename_map.insert(#name.to_owned(), #value.to_owned());
        }
            .to_string());
    }
    rename_map_block.append("rename_map}");

    if rename_map_exists {
        rename_map_block
    } else {
        quote!{::std::collections::HashMap::new()}
    }
}

fn generate_field_int_map_block(ast: &DeriveInput) -> quote::Tokens {
    let mut field_type_map_block = quote::Tokens::new();
    field_type_map_block.append("{");
    field_type_map_block.append("let mut field_type_map = ::std::collections::HashMap::new();");

    for field in get_ast_struct_fields(ast) {
        let field_ident_string = field.ident.clone().unwrap().to_string();
        let field_type = type_to_string(&field.ty);

        if is_vec(&field_type) && vec_is_struct_type(&field_type) {
            let generic_type = get_vec_generic_type(&field_type);
            field_type_map_block.append(&quote!{
                field_type_map.insert(#field_ident_string.to_owned(),
                    ::arthas::traits::FieldType::Array(#generic_type::get_field_type_map()));
                }
                .to_string());
        } else if is_hashmap(&field_type) && hashmap_is_struct_type(&field_type) {
            let generic_type = get_hashmap_generic_type(&field_type);
            field_type_map_block.append(&quote!{
                field_type_map.insert(#field_ident_string.to_owned(),
                    ::arthas::traits::FieldType::Object(#generic_type::get_field_type_map()));}
                .to_string());
        } else if is_option(&field_type) && option_is_struct_type(&field_type) {
            let generic_type = get_option_generic_type(&field_type);
            field_type_map_block.append(&quote!{
                field_type_map.insert(#field_ident_string.to_owned(),
                    ::arthas::traits::FieldType::Object(#generic_type::get_field_type_map()));}
                .to_string());
        } else if is_struct_type(&field_type) {
            let generic_type = to_ident(field_type);
            field_type_map_block.append(&quote!{
                field_type_map.insert(#field_ident_string.to_owned(),
                    ::arthas::traits::FieldType::Struct(#generic_type::get_field_type_map()));}
                .to_string());
        } else {
            field_type_map_block.append(&quote!{
                field_type_map.insert(#field_ident_string.to_owned(),
                    ::arthas::traits::FieldType::Atomic(#field_type.to_owned()));}
                .to_string());
        }
    }

    field_type_map_block.append("return field_type_map;");
    field_type_map_block.append("}");
    field_type_map_block
}

fn to_ident<I: AsRef<str>>(ident_str: I) -> Ident {
    Ident::from(ident_str.as_ref())
}

fn type_to_string(ty: &syn::Ty) -> String {
    let mut ty_tokens = quote::Tokens::new();
    ty.to_tokens(&mut ty_tokens);
    ty_tokens.to_string()
}

fn check_is_struct(ast: &DeriveInput) {
    if let syn::Body::Struct(_) = ast.body {
    } else {
        panic!("#[derive(Arthas)] is only defined for structs, not for enums or other!");
    }
}

fn check_has_id(ast: &DeriveInput) -> bool {
    for field in get_ast_struct_fields(ast) {
        if let Some(ref ident) = field.ident {
            if ident.to_string() == "_id" {
                return true;
            }
        }
    }

    false
}

fn check_is_one(ast: &DeriveInput) -> bool {
    for attr in &ast.attrs {
        if let syn::MetaItem::List(ref ident, ref meta_items) = attr.value {
            if ident.to_string() == "arthas" {
                for item in meta_items {
                    if let syn::NestedMetaItem::MetaItem(ref meta_item) = *item {
                        if let syn::MetaItem::Word(ref ident) = *meta_item {
                            if ident.to_string() == "is_one" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn is_vec(type_: &str) -> bool {
    type_.starts_with("Vec<") || type_.starts_with("Vec <")
}

fn is_option(type_: &str) -> bool {
    type_.starts_with("Option<") || type_.starts_with("Option <")
}

fn is_hashmap(type_: &str) -> bool {
    type_.starts_with("HashMap<") || type_.starts_with("HashMap <")
}

fn get_vec_generic_type(type_: &str) -> Ident {
    let reg = Regex::new("Vec *< *(.*) *>").unwrap();
    let cap = reg.captures(type_).unwrap();
    to_ident(cap[1].trim())
}

fn get_option_generic_type(type_: &str) -> Ident {
    let reg = Regex::new(r"Option *< *(.*) *>").unwrap();
    let cap = reg.captures(type_).unwrap();
    to_ident(cap[1].trim())
}

fn get_hashmap_generic_type(type_: &str) -> Ident {
    let reg = Regex::new(r"HashMap *< *[a-z0-9A-Z]+ *, *(.*) *>").unwrap();
    let cap = reg.captures(type_).unwrap();
    to_ident(cap[1].trim())
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
