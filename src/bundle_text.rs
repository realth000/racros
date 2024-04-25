use proc_macro::TokenStream;
use std::fs::OpenOptions;
use std::io::Read;
use std::process::Stdio;

use proc_macro2::{Ident, TokenTree};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Meta, MetaList, Path};

use crate::util::compiling_error;

const KEY_NAME: &str = "name";
const KEY_FILE: &str = "file";
const KEY_COMMAND: &str = "command";

const ERROR_INVALID_USAGE: &str = r#"invalid usage of #[derive(BundleText)].
Only support bundling file content or command output.
Example: 
1. Bundle file: #[bundle(name = "get_file", file = "file/path")]
2. Bundle command: #[bundle(name = "get_rustc_version", command = "rustc --version")]
"#;

pub fn bundle_text_internal(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match &ast.data {
        Data::Enum(_) => bundle_text_enum(&ast),
        _ => compiling_error!(
            proc_macro2::Span::call_site(),
            "#derive[(BundleText)] only support enums"
        ),
    }
}

fn bundle_text_enum(ast: &DeriveInput) -> TokenStream {
    let target_ident = &ast.ident;
    let mut ret = vec![];
    for attr in &ast.attrs {
        if let Attribute {
            meta:
                Meta::List(MetaList {
                    path: Path { segments, .. },
                    tokens: ts, // proc_macro2::TokenStream { .. },
                    ..
                }),
            ..
        } = attr
        {
            if segments.is_empty() || segments.last().unwrap().ident != "bundle" {
                continue;
            }
            let mut keys = vec![];
            let mut values = vec![];
            for tt in ts.into_token_stream() {
                match tt {
                    TokenTree::Ident(ident) => keys.push(ident),
                    TokenTree::Literal(literal) => values.push(literal),
                    _ => continue,
                }
            }

            // Catch and check these legal values:
            //
            // 1. `#[bundle(name = "get_file", file = "file/path")]`
            // 2. `#[bundle(name = "get_rustc_version", command = "rustc --version")]`
            if keys.len() != 2
                || values.len() != 2
                || keys[0] != KEY_NAME
                || (keys[1] != KEY_FILE && keys[1] != KEY_COMMAND)
            {
                return compiling_error!(proc_macro2::Span::call_site(), "{ERROR_INVALID_USAGE}");
            }
            let v1s = values[0].to_string();
            let name = Ident::new(v1s.get(1..v1s.len() - 1).unwrap(), values[0].span());
            let content = match keys[1].to_string().as_str() {
                KEY_FILE => {
                    // Surrounded by double quote.
                    let raw_file_name = values[1].to_string();
                    let file_name = raw_file_name.get(1..raw_file_name.len() - 1).unwrap();
                    let mut file_path = std::env::current_dir().unwrap();
                    file_path.push(file_name);
                    let file = OpenOptions::new().read(true).open(&file_path);
                    if file.is_err() {
                        return compiling_error!(
                            proc_macro2::Span::call_site(),
                            "failed to open file {file_path:#?}: {}",
                            file.err().unwrap()
                        );
                    }
                    let mut file_content = String::new();
                    if let Err(e) = file.unwrap().read_to_string(&mut file_content) {
                        return compiling_error!(
                            proc_macro2::Span::call_site(),
                            "failed to read file {file_path:#?}: {}",
                            e,
                        );
                    }
                    file_content
                }
                KEY_COMMAND => {
                    let v1s = values[1].to_string();

                    let mut command_and_args = v1s
                        .get(1..v1s.len() - 1)
                        .unwrap()
                        .split(" ")
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>();
                    let command = command_and_args.remove(0);
                    let child = std::process::Command::new(command)
                        .args(command_and_args)
                        .stdout(Stdio::piped())
                        .spawn();
                    if let Err(e) = child {
                        return compiling_error!(
                            proc_macro2::Span::call_site(),
                            "failed to run command {}: {}",
                            values[1],
                            e,
                        );
                    }
                    let mut command_output = String::new();
                    let mut x = child.unwrap().stdout.unwrap();
                    let run_result = x.read_to_string(&mut command_output);
                    if run_result.is_err() {
                        return compiling_error!(
                            proc_macro2::Span::call_site(),
                            "failed to run command {}: {}",
                            values[1],
                            run_result.err().unwrap()
                        );
                    }

                    command_output
                }
                _ => panic!("impossible"),
            };

            let expand = quote! {
                fn #name() -> &'static str {
                    #content
                }
            };

            ret.push(expand);
        }
    }
    let expand = quote! {
        impl #target_ident {
            #(#ret)*
        }
    };
    expand.into()
}
