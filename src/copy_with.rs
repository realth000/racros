use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Field, Meta, Path, Type, TypePath};

use crate::util::compiling_error;

pub fn copy_with_internal(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    // println!("{:#?}", ast);
    let Data::Struct(struct_enum) = &ast.data else {
        return compiling_error!(
            proc_macro2::Span::call_site(),
            "#[derive(CopyWith)] only support structs"
        );
    };
    let target_ident = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let mut field_copy_vec: Vec<proc_macro2::TokenStream> = vec![];

    for field in &struct_enum.fields {
        if let Field {
            ident: Some(ident),
            ty,
            ..
        } = &field
        {
            // When using `#[copy]` attribute, use `self.field.copy_with(other.field)`.
            let mut use_copy = false;

            for attr in &field.attrs {
                if let Attribute {
                    meta: Meta::Path(Path { segments, .. }),
                    ..
                } = &attr
                {
                    if segments.last().as_ref().unwrap().ident == "copy" {
                        use_copy = true;
                    }
                }
            }

            let segments = match &ty {
                Type::Path(TypePath {
                    path: Path { segments, .. },
                    ..
                }) => segments,
                Type::Reference(_) => {
                    return compiling_error!(ident.span(), "struct has reference fields can not #[derive(CopyWith)] because copied data is not borrowed");
                }
                _ => {
                    return compiling_error!(ident.span(), "only TypePath is allowed here");
                }
            };
            if use_copy {
                field_copy_vec.push(quote! {
                    self.#ident.copy_with(&other.#ident);
                });
            } else {
                let path_ident = &segments.last().as_ref().unwrap().ident;
                field_copy_vec.push(quote! {
                    if other.#ident != #path_ident::default() {
                        self.#ident = other.#ident.clone();
                    }
                });
            }
        }
    }

    let expand = quote! {
       impl #impl_generics #target_ident #ty_generics #where_clause {
            fn copy_with(&mut self, other: &Self) {
                #(#field_copy_vec)*
            }
        }
    };

    expand.into()
}
