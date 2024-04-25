use crate::util::compiling_error;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Punct, TokenTree};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Meta, MetaList, Path};

static KEY_NAME: &str = "name";
static KEY_FILE: &str = "file";
static KEY_COMMAND: &str = "command";

static ERROR_INVALID_USAGE: &str = r#"invalid usage of #[derive(BundleText)].
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

/// Describe and parse the bundlers in attributes.
///
/// # Usage
///
/// ## Bundle file content
///
/// `#[bundle(name = "get_file", file = "file/path")]`
///
/// ## Bundle command output
///
/// `#[bundle(name = "get_rustc_version", command = "rustc --version")]`
#[derive(Debug)]
struct SingleBundler {
    name: Ident,
    punct: Punct,
    name_value: Literal,
    punct2: Punct,
    source: Ident,
    punct3: Punct,
    source_value: Literal,
}

impl Parse for SingleBundler {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            punct: input.parse()?,
            name_value: input.parse()?,
            punct2: input.parse()?,
            source: input.parse()?,
            punct3: input.parse()?,
            source_value: input.parse()?,
        })
    }
}

fn bundle_text_enum(ast: &DeriveInput) -> TokenStream {
    let mut keys = vec![];
    let mut values = vec![];
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
            for tt in ts.into_token_stream() {
                match tt {
                    TokenTree::Ident(ident) => keys.push(ident),
                    TokenTree::Literal(literal) => values.push(literal),
                    _ => continue,
                }
            }

            break;
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
    panic!("keys={keys:#?}, values={values:#?}, {:#?}", keys.len());
    TokenStream::new()
}
