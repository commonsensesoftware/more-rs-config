use crate::gen::{collect_type_param_idents, StructDef};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{GenericParam, Ident};

/// Generate the full `fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>`
/// method body for a named struct.
///
/// The generated code:
/// 1. Defines a `__Field` enum with variants for each non-skipped field plus `__Ignore`
/// 2. Implements `Deserialize` for `__Field` to match string keys (including aliases)
/// 3. Defines a `__Visitor` struct implementing `serde::de::Visitor` with `visit_map`
/// 4. In `visit_map`: accumulates `Option<FieldType>` locals, matches keys, deserializes values
/// 5. After map exhaustion: resolves each field (value, default, or missing_field error)
/// 6. Skipped fields use `Default::default()` or the field's default expression
/// 7. Unknown keys are consumed via `IgnoredAny`
pub fn deserialize(struct_def: &StructDef) -> TokenStream {
    let struct_ident = &struct_def.ident;
    let struct_name_str = struct_ident.to_string();
    let (_impl_generics, ty_generics, _where_clause) = struct_def.generics.split_for_impl();

    // separate fields into non-skipped (deserialized) and skipped
    let deserialized_fields: Vec<_> = struct_def
        .fields
        .iter()
        .filter(|f| !f.attrs.skip_deserializing)
        .collect();

    let skipped_fields: Vec<_> = struct_def
        .fields
        .iter()
        .filter(|f| f.attrs.skip_deserializing)
        .collect();

    // --- generate __Field enum variants ---
    let field_enum_variants: Vec<_> = deserialized_fields
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("__field{}", i))
        .collect();

    // --- generate match arms for __Field deserialization ---
    let field_match_arms: Vec<TokenStream> = deserialized_fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let variant = &field_enum_variants[i];
            let primary_name = &field.attrs.serialized_name;

            // Collect all names this field responds to: primary + aliases
            let mut all_names: Vec<&str> = vec![primary_name.as_str()];
            for alias in &field.attrs.aliases {
                all_names.push(alias.as_str());
            }

            quote! {
                #(#all_names)|* => Ok(__Field::#variant),
            }
        })
        .collect();

    // --- generate Option<T> locals for visit_map ---
    let option_locals: Vec<TokenStream> = deserialized_fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let local_name = format_ident!("__field{}", i);
            let ty = &field.ty;
            quote! {
                let mut #local_name: Option<#ty> = None;
            }
        })
        .collect();

    // --- generate key match arms in visit_map ---
    let visit_map_match_arms: Vec<TokenStream> = deserialized_fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let variant = &field_enum_variants[i];
            let local_name = format_ident!("__field{}", i);
            let ty = &field.ty;
            quote! {
                __Field::#variant => {
                    if #local_name.is_some() {
                        return Err(serde::de::Error::duplicate_field(FIELDS[#i]));
                    }
                    #local_name = Some(map.next_value::<#ty>()?);
                }
            }
        })
        .collect();

    // --- generate field resolution after map exhaustion ---
    let field_resolutions: Vec<TokenStream> = deserialized_fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let local_name = format_ident!("__field{}", i);
            let field_ident = &field.ident;
            let serialized_name = &field.attrs.serialized_name;

            if field.attrs.has_field_default {
                // field has its own #[serde(default)] or #[serde(default = "path")]
                if let Some(ref default_expr) = field.attrs.default_expr {
                    quote! {
                        let #field_ident = #local_name.unwrap_or_else(|| #default_expr);
                    }
                } else {
                    quote! {
                        let #field_ident = #local_name.unwrap_or_default();
                    }
                }
            } else if struct_def.container_attrs.has_default {
                // container has #[serde(default)] — use Default::default() for absent fields
                let ty = &field.ty;
                quote! {
                    let #field_ident = #local_name.unwrap_or_else(|| <#ty as Default>::default());
                }
            } else {
                // required field — return missing_field error if absent
                quote! {
                    let #field_ident = #local_name.ok_or_else(|| {
                        serde::de::Error::missing_field(#serialized_name)
                    })?;
                }
            }
        })
        .collect();

    // --- generate skipped field defaults ---
    let skipped_field_defaults: Vec<TokenStream> = skipped_fields
        .iter()
        .map(|field| {
            let field_ident = &field.ident;
            if let Some(ref default_expr) = field.attrs.default_expr {
                quote! {
                    let #field_ident = #default_expr;
                }
            } else {
                let ty = &field.ty;
                quote! {
                    let #field_ident = <#ty as Default>::default();
                }
            }
        })
        .collect();

    // --- generate struct construction ---
    let all_field_idents: Vec<_> = struct_def.fields.iter().map(|f| &f.ident).collect();

    // --- FIELDS constant for error messages ---
    let field_names_strs: Vec<&str> = deserialized_fields
        .iter()
        .map(|f| f.attrs.serialized_name.as_str())
        .collect();

    // --- generate the expecting message ---
    let expecting_msg = format!("struct {}", struct_name_str);

    // --- generate visitor struct and impl based on whether the struct is generic ---
    let has_generics = !struct_def.generics.params.is_empty();

    let (visitor_def, visitor_impl, visitor_instantiation) = if has_generics {
        // For generic structs, the visitor needs type parameters and PhantomData.
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

        // build struct definition params with existing bounds (needed when the struct
        // itself has bounds, e.g. `struct Foo<T: Default>` — using `Foo<T>` in a field
        // requires `T: Default` to be satisfied)
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

        // build bounds for the visitor impl: existing bounds + Deserialize<'de> only for
        // type parameters that appear in deserializable fields.
        let deserializable_type_params: std::collections::HashSet<&Ident> = deserialized_fields
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
            struct __Visitor<#(#struct_def_params),*>(core::marker::PhantomData<(#(#type_param_idents,)*)>);
        };

        let visitor_impl = quote! {
            impl<'de, #(#visitor_impl_bounds),*> serde::de::Visitor<'de> for __Visitor<#(#type_param_idents),*> {
                type Value = #struct_ident #ty_generics;

                fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    f.write_str(#expecting_msg)
                }

                fn visit_map<__A>(self, mut map: __A) -> Result<Self::Value, __A::Error>
                where
                    __A: serde::de::MapAccess<'de>,
                {
                    #(#option_locals)*

                    while let Some(key) = map.next_key::<__Field>()? {
                        match key {
                            #(#visit_map_match_arms)*
                            __Field::__ignore => {
                                let _ = map.next_value::<serde::de::IgnoredAny>()?;
                            }
                        }
                    }

                    #(#field_resolutions)*
                    #(#skipped_field_defaults)*

                    Ok(#struct_ident {
                        #(#all_field_idents,)*
                    })
                }
            }
        };

        let visitor_instantiation = quote! {
            __Visitor::<#(#type_param_idents),*>(core::marker::PhantomData)
        };

        (visitor_def, visitor_impl, visitor_instantiation)
    } else {
        // for non-generic structs, use a simple unit struct visitor
        let visitor_def = quote! {
            struct __Visitor;
        };

        let visitor_impl = quote! {
            impl<'de> serde::de::Visitor<'de> for __Visitor {
                type Value = #struct_ident;

                fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    f.write_str(#expecting_msg)
                }

                fn visit_map<__A>(self, mut map: __A) -> Result<Self::Value, __A::Error>
                where
                    __A: serde::de::MapAccess<'de>,
                {
                    #(#option_locals)*

                    while let Some(key) = map.next_key::<__Field>()? {
                        match key {
                            #(#visit_map_match_arms)*
                            __Field::__ignore => {
                                let _ = map.next_value::<serde::de::IgnoredAny>()?;
                            }
                        }
                    }

                    #(#field_resolutions)*
                    #(#skipped_field_defaults)*

                    Ok(#struct_ident {
                        #(#all_field_idents,)*
                    })
                }
            }
        };

        let visitor_instantiation = quote! {
            __Visitor
        };

        (visitor_def, visitor_impl, visitor_instantiation)
    };

    // --- assemble the full deserialize method ---
    quote! {
        fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
        where
            __D: serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                #(#field_enum_variants,)*
                __ignore,
            }

            impl<'de> serde::Deserialize<'de> for __Field {
                fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
                where
                    __D: serde::Deserializer<'de>,
                {
                    struct __FieldVisitor;

                    impl<'de> serde::de::Visitor<'de> for __FieldVisitor {
                        type Value = __Field;

                        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                            f.write_str("field identifier")
                        }

                        fn visit_str<__E>(self, value: &str) -> Result<__Field, __E>
                        where
                            __E: serde::de::Error,
                        {
                            match value {
                                #(#field_match_arms)*
                                _ => Ok(__Field::__ignore),
                            }
                        }
                    }

                    deserializer.deserialize_identifier(__FieldVisitor)
                }
            }

            #visitor_def

            #visitor_impl

            const FIELDS: &[&str] = &[#(#field_names_strs),*];
            deserializer.deserialize_struct(#struct_name_str, FIELDS, #visitor_instantiation)
        }
    }
}
