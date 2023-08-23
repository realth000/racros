use crate::util::compiling_error;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, ExprLit, Field, Lit, Meta, MetaNameValue,
};

enum DebugStyle {
    Struct,
    Tuple,
}

enum DebugFormat {
    Debug,
    Display,
}

pub fn auto_debug_internal(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", ast);
    let data_struct = if let Data::Struct(data_struct) = &ast.data {
        data_struct
    } else {
        return compiling_error!(
            proc_macro2::Span::call_site(),
            "#derive[(AutoDebug)] only supports struct",
        );
    };

    let debug_placeholder = "{:#?}";
    let display_placeholder = "{}";

    let mut debug_style = DebugStyle::Struct;
    let mut debug_format = DebugFormat::Debug;

    let target_ident = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let mut field_vec: Vec<proc_macro2::TokenStream> = vec![];

    // Check attributes on struct.
    for attr in &ast.attrs {
        if let Attribute {
            meta:
                Meta::NameValue(MetaNameValue {
                    path,
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(token, ..),
                            ..
                        }),
                    ..
                }),
            ..
        } = attr
        {
            if path.segments.is_empty() {
                continue;
            }

            match path.segments.last().unwrap().ident.to_string().as_str() {
                "debug_style" => {
                    debug_style = match check_debug_style(token.token().to_string().as_str()) {
                        Some(v) => v,
                        None => {
                            return compiling_error!(token.span(), "invalid debug_style");
                        }
                    }
                }
                "debug_format" => {
                    debug_format = match check_debug_format(token.token().to_string().as_str()) {
                        Some(v) => v,
                        None => {
                            return compiling_error!(token.span(), "invalid debug_format");
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    let target_ident_str = target_ident.to_string();

    let field_vec_header = match debug_style {
        DebugStyle::Struct => quote! {
            let mut ff = f.debug_struct(#target_ident_str);
        },
        DebugStyle::Tuple => quote! {
            let mut ff = f.debug_tuple(#target_ident_str);
        },
    };

    for field in &data_struct.fields {
        let mut field_override_name: Option<String> = None;
        let mut field_override_value: Option<String> = None;
        let mut field_ignore = false;
        let mut field_format: Option<DebugFormat> = None;
        if let Field {
            ident: Some(field_ident),
            attrs,
            ..
        } = &field
        {
            // Check field attr.
            for attr in attrs {
                if let Attribute {
                    meta:
                        Meta::NameValue(MetaNameValue {
                            path,
                            value:
                                Expr::Lit(ExprLit {
                                    lit: Lit::Str(token, ..),
                                    ..
                                }),
                            ..
                        }),
                    ..
                } = &attr
                {
                    if path.segments.is_empty() {
                        continue;
                    }

                    match path.segments.last().unwrap().ident.to_string().as_str() {
                        "debug_name" => {
                            field_override_name =
                                Some(token.token().to_string().trim_matches('"').to_string())
                        }
                        "debug_value" => {
                            field_override_value =
                                Some(token.token().to_string().trim_matches('"').to_string())
                        }
                        _ => continue,
                    }
                } else if let Attribute {
                    meta: Meta::Path(path),
                    ..
                } = &attr
                {
                    match path.segments.last().unwrap().ident.to_string().as_str() {
                        "debug_ignore" => field_ignore = true,
                        "debug_display" => field_format = Some(DebugFormat::Display),
                        "debug_debug" => field_format = Some(DebugFormat::Debug),
                        _ => continue,
                    }
                }
            }

            if field_ignore {
                continue;
            }

            let field_debug_name = match field_override_name {
                Some(v) => v,
                None => field_ident.to_string(),
            };

            let field_placeholder = match field_format {
                Some(DebugFormat::Debug) => debug_placeholder,
                Some(DebugFormat::Display) => display_placeholder,
                None => match debug_format {
                    DebugFormat::Debug => debug_placeholder,
                    DebugFormat::Display => display_placeholder,
                },
            };

            let field_value = match field_override_value {
                Some(v) => quote! {#v},
                None => quote! {self.#field_ident},
            };

            match debug_style {
                DebugStyle::Struct => field_vec.push(quote! {
                    ff.field(#field_debug_name, &format_args!(#field_placeholder, #field_value));
                }),
                DebugStyle::Tuple => field_vec.push(quote! {
                    ff.field(&format_args!(#field_placeholder, #field_value));
                }),
            }
        } else {
            // TODO: Check skip unnamed fields.
            continue;
        }
    }

    let expand = quote! {
        impl #impl_generics std::fmt::Debug for #target_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #field_vec_header
                #(#field_vec)*
                ff.finish()
            }
        }
    };

    expand.into()
}

fn check_debug_style(style: &str) -> Option<DebugStyle> {
    match style.trim_matches('"') {
        "struct" => Some(DebugStyle::Struct),
        "tuple" => Some(DebugStyle::Tuple),
        _ => None,
    }
}

fn check_debug_format(style: &str) -> Option<DebugFormat> {
    match style.trim_matches('"') {
        "debug" => Some(DebugFormat::Debug),
        "display" => Some(DebugFormat::Display),
        _ => None,
    }
}
