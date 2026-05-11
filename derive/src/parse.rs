use syn::{Data, DeriveInput, Fields, FieldsNamed};

/// Validates that the derive input is a struct with named fields.
///
/// Returns the named fields on success, or a compile error if the input
/// is an enum, tuple struct, or unit struct.
pub fn validate_input(input: &DeriveInput) -> syn::Result<&FieldsNamed> {
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => Ok(fields),
            _ => Err(syn::Error::new_spanned(
                &input.ident,
                "config::Deserialize can only be derived for structs with named fields",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "config::Deserialize can only be derived for structs with named fields",
        )),
    }
}
