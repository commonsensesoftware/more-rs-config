mod attr;
mod gen;
mod parse;

use gen::{FieldDef, StructDef};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericParam, Ident, Result};

/// Derive macro that generates a custom `impl serde::Deserialize<'de>` for named structs.
///
/// # Remarks
///
/// Unlike serde's default derive, this macro produces a `deserialize_in_place()` method that only updates fields
/// present in the deserializer's map, leaving absent fields at their current values.
#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    match derive_deserialize_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_deserialize_impl(input: TokenStream) -> Result<TokenStream> {
    // 1. parse the input TokenStream into a DeriveInput
    let input: DeriveInput = syn::parse(input)?;

    // 2. validate that the input is a named struct
    let _fields = parse::validate_input(&input)?;

    // 3. parse serde attributes from the struct
    let parsed = attr::parse_struct_attrs(&input)?;

    // 4. build the StructDef for code generation
    let struct_def = StructDef {
        ident: input.ident,
        generics: input.generics,
        container_attrs: parsed.container_attrs,
        fields: parsed
            .fields
            .into_iter()
            .map(|f| FieldDef {
                ident: f.ident,
                ty: f.ty,
                attrs: f.attrs,
            })
            .collect(),
    };

    // 5. generate the deserialize method
    let deserialize_body = gen::deserialize(&struct_def);

    // 6. generate the deserialize_in_place method
    let deserialize_in_place_body = gen::deserialize_in_place(&struct_def);

    // 7. build the full impl block
    let struct_ident = &struct_def.ident;
    let (_impl_generics, ty_generics, where_clause) = struct_def.generics.split_for_impl();

    // for non-generic structs, we just need `impl<'de>`. for generic structs, we add `T: serde::Deserialize<'de>`
    // bounds only for type parameters that appear in deserializable (non-skipped) fields
    let generic_params = &struct_def.generics.params;
    let output = if generic_params.is_empty() {
        quote! {
            #[automatically_derived]
            impl<'de> serde::Deserialize<'de> for #struct_ident #where_clause {
                #deserialize_body

                #deserialize_in_place_body
            }
        }
    } else {
        // collect type parameter identifiers that appear in deserializable fields
        let deserializable_type_params: std::collections::HashSet<Ident> = struct_def
            .fields
            .iter()
            .filter(|f| !f.attrs.skip_deserializing)
            .flat_map(|f| extract_type_params(&f.ty, &struct_def.generics))
            .collect();

        // build impl generics with Deserialize<'de> bounds only on type parameters used in deserializable fields
        let impl_params = generic_params.iter().map(|param| match param {
            GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                let existing_bounds = &type_param.bounds;
                let needs_deserialize = deserializable_type_params.contains(ident);
                match (existing_bounds.is_empty(), needs_deserialize) {
                    (true, true) => quote! { #ident: serde::Deserialize<'de> },
                    (true, false) => quote! { #ident },
                    (false, true) => quote! { #ident: #existing_bounds + serde::Deserialize<'de> },
                    (false, false) => quote! { #ident: #existing_bounds },
                }
            }
            GenericParam::Lifetime(lt) => quote! { #lt },
            GenericParam::Const(cp) => quote! { #cp },
        });

        // build the where clause, combining existing predicates with any additional ones
        let where_clause_output = if let Some(wc) = where_clause {
            quote! { #wc }
        } else {
            quote! {}
        };

        quote! {
            #[automatically_derived]
            impl<'de, #(#impl_params),*> serde::Deserialize<'de> for #struct_ident #ty_generics #where_clause_output {
                #deserialize_body

                #deserialize_in_place_body
            }
        }
    };

    Ok(output.into())
}

/// Extracts type parameter identifiers from a type that match the struct's generic parameters.
/// For example, given `Vec<T>` and generics `<T, U>`, this returns `{T}`.
fn extract_type_params(ty: &syn::Type, generics: &syn::Generics) -> Vec<syn::Ident> {
    let type_param_idents: Vec<&syn::Ident> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(tp) => Some(&tp.ident),
            _ => None,
        })
        .collect();

    let mut found = Vec::new();
    gen::collect_type_param_idents(ty, &type_param_idents, &mut found);
    found.into_iter().cloned().collect()
}
