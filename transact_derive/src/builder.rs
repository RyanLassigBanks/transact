// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate proc_macro;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    Attribute, Data, DeriveInput, Error as SynError, Field, Fields, GenericArgument, Ident, Lit,
    Meta, PathArguments, Type,
};

pub fn generate_builder_macro(derive_input: DeriveInput) -> Result<TokenStream2, SynError> {
    let struct_impl = generate_struct_impl(derive_input.clone())?;
    let builder_struct = generate_builder_struct(derive_input.clone())?;

    Ok(quote! {
        #struct_impl
        #builder_struct
    })
}

fn generate_struct_impl(derive_input: DeriveInput) -> Result<TokenStream2, SynError> {
    let struct_name = derive_input.ident.clone();
    let getters = generate_getters(get_struct_fields(derive_input.clone())?)?;

    Ok(quote! {
        impl #struct_name {
            #(#getters)*
        }
    })
}

fn generate_builder_struct(derive_input: DeriveInput) -> Result<TokenStream2, SynError> {
    let builder_name = generate_builder_name(derive_input.clone());
    let mut setters = Vec::new();
    let mut field_names = Vec::new();

    for field in get_struct_fields(derive_input.clone())?.iter() {
        let field_name = field.ident.clone().unwrap();
        let ty = field.ty.clone();

        let setter_name = Ident::new(
            &format!("with_{}", field.ident.clone().unwrap().to_string()),
            Span::call_site(),
        );

        setters.push(quote! {
            pub fn #setter_name(mut self, value: #ty) -> Self {
                self.#field_name = Some(value);
                self
            }
        });

        field_names.push(quote! {
            #field_name: Option<#ty>
        });
    }

    let build_impl = if has_gen_build_fn_attr(&derive_input.attrs) {
        generate_build_impl(derive_input.clone())?
    } else {
        quote!()
    };

    Ok(quote! {
        #[derive(Default, Clone)]
        pub struct #builder_name {
            #(#field_names), *
        }

        impl #builder_name {
            pub fn new() -> Self {
                #builder_name::default()
            }

            #(#setters)*
        }

        #build_impl
    })
}

fn generate_build_impl(derive_input: DeriveInput) -> Result<TokenStream2, SynError> {
    let struct_name = derive_input.ident.clone();
    let builder_name = generate_builder_name(derive_input.clone());
    let fields = get_struct_fields(derive_input.clone())?;
    let mut let_stmts = Vec::new();
    let mut field_names = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.clone().unwrap();

        let let_stmt = if has_optional_attr(field) {
            quote! {
                let #field_name = self.#field_name.unwrap_or_default();
            }
        } else {
            quote! {
                let #field_name = self.#field_name.ok_or_else(|| BuilderError::MissingField(stringify!(#field_name).into()))?;
            }
        };

        let_stmts.push(let_stmt);
        field_names.push(field_name);
    }

    Ok(quote! {
        impl Build for #builder_name {
            type Result = Result<#struct_name, BuilderError>;

            fn build(self) -> Self::Result {
                #(#let_stmts)*

                Ok(#struct_name {
                    #(#field_names), *
                })
            }
        }
    })
}

fn get_struct_fields(derive_input: DeriveInput) -> Result<Fields, SynError> {
    if let Data::Struct(d) = derive_input.data {
        Ok(d.fields)
    } else {
        Err(SynError::new_spanned(
            derive_input.into_token_stream(),
            "builder is only compatible with stucts",
        ))
    }
}

fn generate_getters(fields: Fields) -> Result<Vec<TokenStream2>, SynError> {
    let mut tokens = Vec::new();

    for field in fields.iter().filter(|field| has_getter_attr(field)) {
        let name = field.ident.clone().unwrap();
        let ty = field.ty.clone();

        if is_string(&ty) {
            tokens.push(quote! {
                pub fn #name(&self) -> &str {
                    &self.#name
                }
            });
        } else if is_vec(&ty) {
            let ty = extract_type_from_generic(&ty)?;
            tokens.push(quote! {
                pub fn #name(&self) -> &[#ty] {
                    &self.#name
                }
            });
        } else {
            tokens.push(quote! {
                pub fn #name(&self) -> &#ty {
                    &self.#name
                }
            });
        }
    }

    Ok(tokens)
}

fn has_getter_attr(field: &Field) -> bool {
    has_helper_attribue(field, Ident::new("getter", Span::call_site()))
}

fn has_optional_attr(field: &Field) -> bool {
    has_helper_attribue(field, Ident::new("optional", Span::call_site()))
}

fn has_helper_attribue(field: &Field, ident: Ident) -> bool {
    field
        .attrs
        .iter()
        .filter_map(|attr| {
            if let Ok(meta) = attr.parse_meta() {
                Some(meta)
            } else {
                None
            }
        })
        .any(|meta| meta.name() == ident)
}

fn is_string(ty: &Type) -> bool {
    is_type(Ident::new("String", Span::call_site()), ty)
}

fn is_vec(ty: &Type) -> bool {
    is_type(Ident::new("Vec", Span::call_site()), ty)
}

fn is_type(ident: Ident, ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path.path.segments.iter().any(|x| x.ident == ident)
    } else {
        false
    }
}

fn extract_type_from_generic(ty: &Type) -> Result<Type, SynError> {
    let segment = if let Type::Path(type_path) = ty {
        type_path.path.segments.first()
    } else {
        return Err(SynError::new_spanned(
            ty.into_token_stream(),
            "Type does not have generic",
        ));
    };

    let args = if let Some(seg) = segment {
        seg.into_value().arguments.clone()
    } else {
        return Err(SynError::new_spanned(
            ty.into_token_stream(),
            "Type does not have generic",
        ));
    };

    let angled_bracket_args = if let PathArguments::AngleBracketed(args) = args {
        if let Some(angled_bracket_args) = args.args.first() {
            angled_bracket_args.into_value().clone()
        } else {
            return Err(SynError::new_spanned(
                ty.into_token_stream(),
                "Type does not have generic",
            ));
        }
    } else {
        return Err(SynError::new_spanned(
            ty.into_token_stream(),
            "Type does not have generic",
        ));
    };

    if let GenericArgument::Type(t) = angled_bracket_args {
        Ok(t)
    } else {
        return Err(SynError::new_spanned(
            ty.into_token_stream(),
            "Type does not have generic",
        ));
    }
}

fn generate_builder_name(derive_input: DeriveInput) -> Ident {
    for attr in derive_input.attrs {
        if let Some(name) = extract_builder_name(&attr) {
            return name;
        }
    }

    let struct_name = derive_input.ident.clone();

    Ident::new(
        &format!("{}Builder", struct_name.to_string()),
        Span::call_site(),
    )
}

fn extract_builder_name(attr: &Attribute) -> Option<Ident> {
    let segment = if let Some(segment) = attr.path.segments.first() {
        segment
    } else {
        return None;
    };

    if segment.into_value().ident != Ident::new("builder_name", Span::call_site()) {
        return None;
    }

    let meta_name_value = if let Ok(Meta::NameValue(nv)) = attr.parse_meta() {
        nv
    } else {
        return None;
    };

    if let Lit::Str(s) = meta_name_value.lit {
        Some(Ident::new(&s.value(), Span::call_site()))
    } else {
        None
    }
}

fn has_gen_build_fn_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        let segment = if let Some(segment) = attr.path.segments.first() {
            segment
        } else {
            return false;
        };

        if segment.into_value().ident == Ident::new("gen_build_impl", Span::call_site()) {
            return true;
        }
    }

    false
}
