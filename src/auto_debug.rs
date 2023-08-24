use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Expr, ExprLit, Field,
    Fields, Lit, Meta, MetaNameValue,
};

use crate::util::compiling_error;

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
    match &ast.data {
        Data::Struct(v) => auto_debug_struct(&ast, v),
        Data::Enum(v) => auto_debug_enum(&ast, v),
        Data::Union(_) => {
            return compiling_error!(
                proc_macro2::Span::call_site(),
                "#derive[(AutoDebug)] does not support union",
            );
        }
    }
}

fn auto_debug_struct(ast: &DeriveInput, data_struct: &DataStruct) -> TokenStream {
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

fn auto_debug_enum(ast: &DeriveInput, data_enum: &DataEnum) -> TokenStream {
    let debug_placeholder = "{:#?}";
    let display_placeholder = "{}";

    let mut debug_format = DebugFormat::Debug;

    let target_ident = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let mut variant_vec: Vec<proc_macro2::TokenStream> = vec![];

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

    for variant in &data_enum.variants {
        let mut variant_ignore = false;
        let mut variant_format: Option<DebugFormat> = None;

        let variant_ident = &variant.ident;
        let variant_ident_str = variant_ident.to_string();

        // Check field attr.
        for attr in &variant.attrs {
            if let Attribute {
                meta: Meta::Path(path),
                ..
            } = &attr
            {
                match path.segments.last().unwrap().ident.to_string().as_str() {
                    "debug_ignore" => variant_ignore = true,
                    "debug_display" => variant_format = Some(DebugFormat::Display),
                    "debug_debug" => variant_format = Some(DebugFormat::Debug),
                    _ => continue,
                }
            }
        }

        if variant_ignore {
            continue;
        }

        let variant_placeholder = match variant_format {
            Some(DebugFormat::Debug) => debug_placeholder,
            Some(DebugFormat::Display) => display_placeholder,
            None => match debug_format {
                DebugFormat::Debug => debug_placeholder,
                DebugFormat::Display => display_placeholder,
            },
        };

        let stmt = match &variant.fields {
            Fields::Unit => quote! {
                #target_ident::#variant_ident => f.write_str(format!(#variant_placeholder, #variant_ident_str).as_str())
            },
            Fields::Unnamed(_) => quote! {
                // This branch is for enum variants that have a unnamed type:
                // Foo(MyType)
                #target_ident::#variant_ident(vv) => f.debug_tuple(#variant_ident_str).field(&format_args!(#variant_placeholder, vv)).finish()
            },
            Fields::Named(fields_names) => {
                // This branch is for enum variants that have a struct:
                // Foo{a: i32, b: u32}

                // Left side of arm statement, each element is a "field_name: v1"
                // Test::T4 { a: __self_0, b: __self_1 } =>
                let mut field_left_vec: Vec<proc_macro2::TokenStream> = vec![];

                // Right side of arm statement, each element is a "key(name).value(value)"
                // f.debug_map(field_name).key(field_1).value(v1).key(field_2).value(v2). ... .finish()
                let mut field_right_vec: Vec<proc_macro2::TokenStream> = vec![];
                for (field_index, field) in fields_names.named.iter().enumerate() {
                    let name = &field.ident.as_ref().unwrap();
                    let name_str = name.to_string();

                    // Fill left side.
                    let fill_ident = Ident::new(format!("v{}", field_index).as_str(), field.span());
                    field_left_vec.push(quote! {
                        #name: #fill_ident
                    });

                    // Fill right side.
                    field_right_vec.push(quote! {
                        key(&format_args!("{}", #name_str)).value(#fill_ident)
                    });
                }

                quote! {
                    #target_ident::#variant_ident{#(#field_left_vec),*} => f.debug_map().#(#field_right_vec).*.finish()
                }
            }
        };

        variant_vec.push(stmt);
    }

    let expand = quote! {
        impl #impl_generics std::fmt::Debug for #target_ident #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#variant_vec,)*
                }
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
