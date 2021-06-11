// Rust flags
#![warn(nonstandard_style)]
#![warn(future_incompatible)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]
#![recursion_limit = "1024"]
// Clippy flags
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

use core::panic;
use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use std::clone::Clone;
use std::convert::AsRef;
use std::convert::Into;
use std::fs;
use std::io;
use std::iter::Extend;
use std::iter::Iterator;
use std::path::Path;
use std::path::PathBuf;
use std::string::ToString;

type Kind = u32;
type Title = String;
type Schema = String;
type FileName = String;

const PATH_TO_SCHEMAS: &str = "../shared/schemas";
const SCHEMA_EXTENSION: &str = ".schema.json";
const EXCLUDE: &[&str] = &["message.schema.json"];

/// Procedural macro for generating enumeration of messages kinds.
#[proc_macro]
pub fn generate_zeromq_messages_kinds_enum(_input: TokenStream) -> TokenStream {
    let schemas_directory_entries_paths = get_schemas_directory_entries_paths(PATH_TO_SCHEMAS)
        .expect("failed to get schemas directory entries path");
    let mut messages_schemas_strings: Vec<Schema> =
        Vec::with_capacity(schemas_directory_entries_paths.len());
    let mut file_name_strings: Vec<FileName> =
        Vec::with_capacity(schemas_directory_entries_paths.len());
    for path in schemas_directory_entries_paths {
        let file_name_string = path
            .file_name()
            .expect("failed to get file name OsStr from path")
            .to_str()
            .expect("failed to get file name as str from OsStr")
            .to_string();
        if (!path.is_dir())
            && file_name_string.ends_with(SCHEMA_EXTENSION)
            && (!EXCLUDE.contains(&file_name_string.as_str()))
        {
            messages_schemas_strings.push(fs::read_to_string(path.clone()).unwrap_or_else(
                |error| panic!("failed to read {:?} content to string: {}", path, error),
            ));

            file_name_strings.push(file_name_string);
        }
    }

    assert_eq!(messages_schemas_strings.len(), file_name_strings.len());

    let mut kinds: Vec<Kind> = Vec::with_capacity(messages_schemas_strings.len());
    let mut titles: Vec<Title> = Vec::with_capacity(messages_schemas_strings.len());
    for current_index in 0..messages_schemas_strings.len() {
        let json: serde_json::Value =
            serde_json::from_str(messages_schemas_strings[current_index].as_str())
                .expect("failed to create json from string");
        #[allow(clippy::cast_possible_truncation)]
        let kind = json
            .get("kind")
            .expect("failed to get kind property from json")
            .as_u64()
            .expect("not integer written in kind property") as Kind;
        kinds.push(kind);

        let title = file_name_strings[current_index]
            .clone()
            .split(SCHEMA_EXTENSION)
            .collect::<Vec<&'_ str>>()[0]
            .to_string();
        titles.push(title);
    }

    assert_eq!(kinds.len(), titles.len());

    let mut variants = quote! {};
    for current_index in 0..kinds.len() {
        let kind_literal = proc_macro2::Literal::u32_unsuffixed(kinds[current_index]);

        let camel_case_title_with_lower_case_first = titles[current_index].to_camel_case();
        let camel_case_title_string = uppercase_first(camel_case_title_with_lower_case_first);
        let syn_title: syn::Variant = syn::parse_str(camel_case_title_string.as_str())
            .expect("failed to parse title into field");

        variants.extend(quote! { #syn_title = #kind_literal, });
    }

    let output = quote! {
        #[repr(u32)]
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, TryFromPrimitive)]
        pub enum ZeromqMessageKind {
            #variants
        }
    };

    output.into()
}

/// Procedural macro for generating messages structs.
#[proc_macro]
pub fn generate_zeromq_messages_structs(_input: TokenStream) -> TokenStream {
    let schemas_directory_entries_paths = get_schemas_directory_entries_paths(PATH_TO_SCHEMAS)
        .expect("failed to get schemas directory entries path");
    let mut file_name_strings: Vec<FileName> =
        Vec::with_capacity(schemas_directory_entries_paths.len());
    let mut paths_to_files: Vec<String> =
        Vec::with_capacity(schemas_directory_entries_paths.len());
    for path in schemas_directory_entries_paths {
        let file_name_string = path
            .file_name()
            .expect("failed to get file name OsStr from path")
            .to_str()
            .expect("failed to get file name as str from OsStr")
            .to_string();

        if (!path.is_dir())
            && file_name_string.ends_with(SCHEMA_EXTENSION)
            && (!EXCLUDE.contains(&file_name_string.as_str()))
        {
            file_name_strings.push(file_name_string);

            paths_to_files.push(
                fs::canonicalize(path)
                    .expect("failed to cannonicalize path")
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }

    assert_eq!(file_name_strings.len(), paths_to_files.len());

    let mut output = quote! {};
    for current_index in 0..file_name_strings.len() {
        let struct_name_with_first_symbol_lower_case = file_name_strings[current_index]
            .split(SCHEMA_EXTENSION)
            .collect::<Vec<&'_ str>>()[0]
            .to_string()
            .to_camel_case();
        let struct_name_string = uppercase_first(struct_name_with_first_symbol_lower_case);
        let struct_name_ident: syn::Ident = syn::parse_str(struct_name_string.as_str())
            .expect("failed to parse struct name str into ident");

        let path_to_schema =
            proc_macro2::Literal::string(paths_to_files[current_index].as_str());

        output.extend(quote! {
            schemafy!(
                root: #struct_name_ident
                #path_to_schema
            );

            #[automatically_derived]
            impl<'de> ZeromqMessageTrait<'de> for #struct_name_ident {
                fn kind() -> ZeromqMessageKind {
                    ZeromqMessageKind::#struct_name_ident
                }
            }
        });
    }

    output.into()
}

fn get_schemas_directory_entries_paths<P: AsRef<Path>>(path: P) -> io::Result<Vec<PathBuf>> {
    let absolute_path = fs::canonicalize(path.as_ref())?;
    let mut schemas_directory_entries_paths = fs::read_dir(absolute_path)?
        .map(|result| result.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    schemas_directory_entries_paths.sort_unstable();

    Ok(schemas_directory_entries_paths)
}

#[allow(clippy::needless_pass_by_value)]
fn uppercase_first(string: String) -> String {
    let mut string_chars = string.chars().collect::<Vec<char>>();
    string_chars[0] = string_chars[0].to_uppercase().next().unwrap();
    string_chars.into_iter().collect::<String>()
}
