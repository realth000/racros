use proc_macro::TokenStream;

use proc_macro2::TokenTree;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, ExprLit, Fields, FieldsUnnamed, Lit,
    Meta, MetaList, MetaNameValue,
};

use crate::util::{compiling_error, to_camel_case, to_pascal_case, to_snake_case};

#[derive(Debug)]
enum Rules {
    Lowercase,
    Uppercase,
    CamelCase,
    PascalCase,
    SnakeCase,
}

pub fn auto_str_internal(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // println!(">>>> ast: {:#?}", &ast);
    let _ = if let Data::Enum(data_enum) = &ast.data {
        data_enum
    } else {
        return compiling_error!(
            proc_macro2::Span::call_site(),
            "#[derive(TryStrFrom)] only support enums"
        );
    };
    // Check default name format:
    // check `#[autorule = "xxx"]`:
    // * Available: lowercase, UPPERCASE, camelCase, PascalCase.
    // * When not set, use the field ident originally.
    let mut rule: Option<Rules> = None;

    for attr in &ast.attrs {
        if let Meta::NameValue(MetaNameValue {
            path,
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Str(token, ..),
                    ..
                }),
            ..
        }) = &attr.meta
        {
            if path.segments.last().unwrap().ident != "autorule" {
                continue;
            }

            rule = match token.token().to_string().trim_matches('"') {
                "lowercase" => Some(Rules::Lowercase),
                "UPPERCASE" => Some(Rules::Uppercase),
                "camelCase" => Some(Rules::CamelCase),
                "PascalCase" => Some(Rules::PascalCase),
                "snake_case" => Some(Rules::SnakeCase),
                _ => {
                    return compiling_error!(
                        token.span(),
                        "unknown AutoStr rules type: {}",
                        token.token()
                    );
                }
            };
            break;
        }
    }

    let mut expand = TokenStream::new();

    let try_from_stream = match generate_try_from(&ast, &rule) {
        Ok(v) => v,
        Err(e) => {
            return e;
        }
    };
    expand.extend(try_from_stream);

    let to_string_stream = match generate_to_string(&ast, &rule) {
        Ok(v) => v,
        Err(e) => {
            return e;
        }
    };
    expand.extend(to_string_stream);

    expand
}

fn generate_try_from(ast: &DeriveInput, rule: &Option<Rules>) -> Result<TokenStream, TokenStream> {
    let target_ident = &ast.ident;

    let mut try_from_arm_vec: Vec<proc_macro2::TokenStream> = vec![];
    let mut try_from_guess_vec: Vec<proc_macro2::TokenStream> = vec![];

    let data_enum = if let Data::Enum(data_enum) = &ast.data {
        data_enum
    } else {
        return Err(compiling_error!(
            proc_macro2::Span::call_site(),
            "#[derive(AutoStr)] only support enums"
        ));
    };

    for variant in &data_enum.variants {
        let field_ident = &variant.ident;

        let mut has_str_attr = false;

        for attr in &variant.attrs {
            if let Attribute {
                meta: Meta::List(MetaList { path, tokens, .. }),
                ..
            } = &attr
            {
                if path.segments.last().unwrap().ident != "str" {
                    continue;
                }
                has_str_attr = true;
                let mut names_vec = vec![];
                tokens.clone().into_iter().for_each(|x| {
                    if let TokenTree::Literal(lit) = &x {
                        names_vec.push(String::from(lit.to_string().trim_matches('"')));
                    }
                });

                let try_from_arm_result = if let Fields::Unnamed(FieldsUnnamed {
                    unnamed, ..
                }) = &variant.fields
                {
                    let f = unnamed.first().unwrap();
                    let wrapped_type = &f.ty;
                    // enum MyEnum {
                    //     #[str=("e1", "e2")]
                    //     E1(MyEnum2),
                    //     E2,
                    // }
                    // "e1, e2" => MyEnum::try_from("e1" or "e2")
                    let mut tmp_vec: Vec<proc_macro2::TokenStream> = vec![];

                    // `value` is the name of arg in `try_from` function signature.
                    // Here should use `value` instead of `#name` (element in names_vec)
                    //
                    // because we want:
                    // "e1" | "e2" => {
                    //     if let Ok(v) = MyEnum::try_from(value) {
                    //         Ok(MyEnum2::E1(v))
                    //     }
                    // }
                    //
                    // not:
                    //
                    // "e1" | "e2" => {
                    //     if let Ok(v) = MyEnum::try_from("e1") {
                    //         Ok(MyEnum2::E1(v))
                    //     }
                    //     if let Ok(v) = MyEnum::try_from("e2") {
                    //         Ok(MyEnum2::E1(v))
                    //     }
                    // }
                    //
                    let target_name_str_ident = target_ident.to_string();
                    tmp_vec.push(quote! {
                        match #wrapped_type::try_from(value) {
                            Ok(v) => Ok(#target_ident::#field_ident(v)),
                            Err(e) => Err(Self::Error::from(format!("failed to convert to {}: {}", #target_name_str_ident, e)))
                        }
                    });
                    tmp_vec.push(quote! {});
                    quote! {#(#tmp_vec)*}
                } else {
                    quote! {
                        Ok(#target_ident::#field_ident)
                    }
                };
                // Add {} around `#try_from_arm_result`, otherwise the compilers seems treating the
                // `if let` statements inside as a serial of sentences, not a block.
                // And that {} can not work if add in names_vec, must add here, where expands.
                //
                // Actually we do not need {} here, because the right side of match arm becomes
                // a single match, not a serial of `if let`.
                try_from_arm_vec.push(quote! {
                    // #(#names_vec)|* => {#try_from_arm_result}
                    #(#names_vec)|* => #try_from_arm_result
                });

                break;
            }
        }

        if !has_str_attr {
            // Do not have a #[str(..)] on this field.
            // Convert from/to string with rule.
            if let Fields::Unnamed(FieldsUnnamed { unnamed, .. }) = &variant.fields {
                let f = unnamed.first().unwrap();
                let wrapped_type = &f.ty;
                let wrapped_type_str = f.ty.to_token_stream().to_string();
                try_from_guess_vec.push(quote! {
                    if let Ok(v) = #wrapped_type::try_from(value) {
                        if fallback_result.is_some() {
                            return Err(Self::Error::from(format!("#[str(...)] attribute not set and fallback guess is ambiguous: both {} and {} can accept this convert", fallback_field.unwrap(), #wrapped_type_str)));
                        }
                        fallback_field = Some(#wrapped_type_str);
                        fallback_result = Some(#target_ident::#field_ident(v));
                    }
                });
            } else {
                let field_ident_str =
                    string_target_with_rule(rule, field_ident.to_string().as_str());
                try_from_arm_vec.push(quote! {
                    #field_ident_str => Ok(#target_ident::#field_ident)
                });
            }
        }
    }

    let target_name_str_ident = target_ident.to_string();

    let guess_block = if try_from_guess_vec.is_empty() {
        quote! {
            Err(Self::Error::from(format!("failed to convert to {} :invalid value", #target_name_str_ident)))
        }
    } else {
        quote! {
            let mut fallback_field : Option<&str> = None;
            let mut fallback_result : Option<Self> = None;
            #(#try_from_guess_vec)*
            match fallback_result {
                Some(v) => Ok(v),
                None => Err(Self::Error::from(format!("failed to convert to {} :invalid value", #target_name_str_ident)))
            }
        }
    };

    let expand = quote! {
        impl TryFrom<&str> for #target_ident {
            type Error = Box<dyn std::error::Error>;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                   #(#try_from_arm_vec,)*
                    _ => {
                        #guess_block
                    },
                }
            }
        }
    };

    Ok(expand.into())
}

fn generate_to_string(ast: &DeriveInput, rule: &Option<Rules>) -> Result<TokenStream, TokenStream> {
    // println!(">>>> ast: {:#?}", &ast);
    let data_enum = if let Data::Enum(data_enum) = &ast.data {
        data_enum
    } else {
        return Err(compiling_error!(
            proc_macro2::Span::call_site(),
            "#[derive(AutoStr)] only support enums"
        ));
    };

    let target_ident = &ast.ident;

    let mut to_string_arm_vec: Vec<proc_macro2::TokenStream> = vec![];

    for variant in &data_enum.variants {
        let field_ident = &variant.ident;

        let mut to_string_target = String::new();
        let mut has_str_attr = false;

        for attr in &variant.attrs {
            if let Attribute {
                meta: Meta::List(MetaList { path, tokens, .. }),
                ..
            } = &attr
            {
                if path.segments.last().unwrap().ident != "str" {
                    continue;
                }
                has_str_attr = true;
                to_string_target =
                    if let Some(TokenTree::Literal(lit)) = &tokens.clone().into_iter().next() {
                        String::from(lit.to_string().trim_matches('"'))
                    } else {
                        variant.ident.to_string().clone()
                    };
                break;
            }
        }

        if has_str_attr {
            match &variant.fields {
                Fields::Unit => {
                    // enum MyEnum {
                    //     E,
                    // }
                    to_string_arm_vec.push(quote! {
                        #target_ident::#field_ident => #to_string_target.to_string()
                    });
                }
                Fields::Unnamed(_) => {
                    // enum MyEnum {
                    //     E(AnotherType),
                    // }
                    //
                    // Call `to_string` on `AnotherType`: MyEnum::E(v) => v.to_string
                    to_string_arm_vec.push(quote! {
                        #target_ident::#field_ident(v) => v.to_string()
                    });
                }
                _ => {}
            }
        } else {
            // Do not have a #[str(..)] on this field.
            // Convert from/to string with rule.
            if let Fields::Unnamed(_) = &variant.fields {
                to_string_arm_vec.push(quote! {
                    #target_ident::#field_ident(v) => v.to_string()
                });
            } else {
                let tmp = string_target_with_rule(
                    rule,
                    String::from(variant.ident.to_string().trim_matches('"')).as_str(),
                );
                to_string_arm_vec.push(quote! {
                    #target_ident::#field_ident => #tmp.to_string()
                });
            }
        }
    }

    let expand = quote! {
        impl ToString for #target_ident {
            fn to_string(&self) -> String {
                match self {
                    #(#to_string_arm_vec,)*
                }
            }
        }
    };

    Ok(expand.into())
}

fn string_target_with_rule(rule: &Option<Rules>, str: &str) -> String {
    match rule {
        Some(Rules::Lowercase) => str.to_lowercase().clone(),
        Some(Rules::Uppercase) => str.to_uppercase().clone(),
        Some(Rules::CamelCase) => to_camel_case(str),
        Some(Rules::PascalCase) => to_pascal_case(str),
        Some(Rules::SnakeCase) => to_snake_case(str),
        None => str.to_string(),
    }
}
