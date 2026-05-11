use crate::gen::{collect_type_param_idents, StructDef};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{GenericParam, Ident};

/// Generate the `fn deserialize_in_place<D: Deserializer<'de>>(deserializer: D, place: &mut Self) -> Result<(), D::Error>`
/// method body for partial-update semantics.
///
/// The generated code:
/// 1. Defines a `__FieldInPlace` enum with variants for each non-skipped field plus `__ignore`
/// 2. Implements `Deserialize` for `__FieldInPlace` matching string keys (including renamed names and aliases)
/// 3. Defines an `__InPlaceVisitor<'a>` struct holding `place: &'a mut StructName`
/// 4. In `visit_map`: matches keys, deserializes values directly into `&mut self.place.field_name`
/// 5. Skipped fields are never matched — treated as unknown
/// 6. Unknown keys are consumed via `next_value::<serde::de::IgnoredAny>()`
pub fn deserialize_in_place(struct_def: &StructDef) -> TokenStream {
    let struct_ident = &struct_def.ident;
    let struct_name_str = struct_ident.to_string();
    let (_impl_generics, ty_generics, _where_clause) = struct_def.generics.split_for_impl();

    // collect non-skipped fields for code generation
    let active_fields: Vec<_> = struct_def
        .fields
        .iter()
        .filter(|f| !f.attrs.skip_deserializing)
        .collect();

    // generate enum variant identifiers for each active field
    let variant_idents: Vec<_> = active_fields
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("__field{}", i))
        .collect();

    // generate match arms for the field key deserializer where each arm matches the serialized name and any aliases
    let field_match_arms: Vec<TokenStream> = active_fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let variant = &variant_idents[i];
            let primary_name = &field.attrs.serialized_name;

            // Collect all names this field responds to: primary + aliases
            let mut all_names: Vec<&str> = vec![primary_name.as_str()];
            for alias in &field.attrs.aliases {
                all_names.push(alias.as_str());
            }

            quote! {
                #(#all_names)|* => Ok(__FieldInPlace::#variant),
            }
        })
        .collect();

    // generate the list of all known field names (for deserialize_struct)
    let field_names_strs: Vec<&str> = active_fields.iter().map(|f| f.attrs.serialized_name.as_str()).collect();

    // generate visit_map match arms that deserialize into place
    let visit_map_arms: Vec<TokenStream> = active_fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let variant = &variant_idents[i];
            let field_ident = &field.ident;
            let ty = &field.ty;
            quote! {
                __FieldInPlace::#variant => {
                    self.place.#field_ident = map.next_value::<#ty>()?;
                }
            }
        })
        .collect();

    // generate the expecting message
    let expecting_msg = format!("struct {}", struct_name_str);

    // generate the __InPlaceVisitor struct and impl based on whether the struct is generic
    let has_generics = !struct_def.generics.params.is_empty();

    let (visitor_def, visitor_impl) = if has_generics {
        let type_params: Vec<_> = struct_def
            .generics
            .params
            .iter()
            .filter_map(|p| match p {
                GenericParam::Type(tp) => Some(tp),
                _ => None,
            })
            .collect();

        let type_param_idents: Vec<_> = type_params.iter().map(|tp| &tp.ident).collect();

        // build struct definition params with existing bounds
        let struct_def_params: Vec<TokenStream> = type_params
            .iter()
            .map(|tp| {
                let ident = &tp.ident;
                let bounds = &tp.bounds;
                if bounds.is_empty() {
                    quote! { #ident }
                } else {
                    quote! { #ident: #bounds }
                }
            })
            .collect();

        // build bounds for the visitor impl: existing bounds + Deserialize<'de> only for type parameters that appear
        // in deserializable fields.
        let deserializable_type_params: std::collections::HashSet<&Ident> = active_fields
            .iter()
            .flat_map(|f| {
                let mut idents = Vec::new();
                collect_type_param_idents(&f.ty, &type_param_idents, &mut idents);
                idents
            })
            .collect();

        let visitor_impl_bounds: Vec<TokenStream> = type_params
            .iter()
            .map(|tp| {
                let ident = &tp.ident;
                let bounds = &tp.bounds;
                let needs_deserialize = deserializable_type_params.contains(ident);
                match (bounds.is_empty(), needs_deserialize) {
                    (true, true) => quote! { #ident: serde::Deserialize<'de> },
                    (true, false) => quote! { #ident },
                    (false, true) => quote! { #ident: #bounds + serde::Deserialize<'de> },
                    (false, false) => quote! { #ident: #bounds },
                }
            })
            .collect();

        let visitor_def = quote! {
            struct __InPlaceVisitor<'__a, #(#struct_def_params),*> {
                place: &'__a mut #struct_ident #ty_generics,
            }
        };

        let visitor_impl = quote! {
            impl<'__a, 'de, #(#visitor_impl_bounds),*> serde::de::Visitor<'de> for __InPlaceVisitor<'__a, #(#type_param_idents),*> {
                type Value = ();

                fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    f.write_str(#expecting_msg)
                }

                #[inline]
                fn visit_map<__A>(self, mut map: __A) -> Result<Self::Value, __A::Error>
                where
                    __A: serde::de::MapAccess<'de>,
                {
                    while let Some(key) = map.next_key::<__FieldInPlace>()? {
                        match key {
                            #( #visit_map_arms )*
                            __FieldInPlace::__ignore => {
                                let _ = map.next_value::<serde::de::IgnoredAny>()?;
                            }
                        }
                    }
                    Ok(())
                }
            }
        };

        (visitor_def, visitor_impl)
    } else {
        let visitor_def = quote! {
            struct __InPlaceVisitor<'__a> {
                place: &'__a mut #struct_ident,
            }
        };

        let visitor_impl = quote! {
            impl<'__a, 'de> serde::de::Visitor<'de> for __InPlaceVisitor<'__a> {
                type Value = ();

                fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    f.write_str(#expecting_msg)
                }

                #[inline]
                fn visit_map<__A>(self, mut map: __A) -> Result<Self::Value, __A::Error>
                where
                    __A: serde::de::MapAccess<'de>,
                {
                    while let Some(key) = map.next_key::<__FieldInPlace>()? {
                        match key {
                            #( #visit_map_arms )*
                            __FieldInPlace::__ignore => {
                                let _ = map.next_value::<serde::de::IgnoredAny>()?;
                            }
                        }
                    }
                    Ok(())
                }
            }
        };

        (visitor_def, visitor_impl)
    };

    // generate the full deserialize_in_place method
    quote! {
        fn deserialize_in_place<__D>(
            __deserializer: __D,
            place: &mut Self,
        ) -> Result<(), __D::Error>
        where
            __D: serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __FieldInPlace {
                #( #variant_idents, )*
                __ignore,
            }

            impl<'de> serde::Deserialize<'de> for __FieldInPlace {
                fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
                where
                    __D: serde::Deserializer<'de>,
                {
                    struct __FieldInPlaceVisitor;

                    impl<'de> serde::de::Visitor<'de> for __FieldInPlaceVisitor {
                        type Value = __FieldInPlace;

                        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                            f.write_str("field identifier")
                        }

                        fn visit_str<__E>(self, value: &str) -> Result<__FieldInPlace, __E>
                        where
                            __E: serde::de::Error,
                        {
                            match value {
                                #( #field_match_arms )*
                                _ => Ok(__FieldInPlace::__ignore),
                            }
                        }
                    }

                    deserializer.deserialize_identifier(__FieldInPlaceVisitor)
                }
            }

            #visitor_def

            #visitor_impl

            const FIELDS: &[&str] = &[#(#field_names_strs),*];
            __deserializer.deserialize_struct(
                #struct_name_str,
                FIELDS,
                __InPlaceVisitor { place },
            )
        }
    }
}
