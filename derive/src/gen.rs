use crate::attr::{ContainerAttrs, FieldAttrs};
use syn::{GenericArgument, Ident, PathArguments, Type};

mod deserialize;
mod in_place;

pub use deserialize::deserialize;
pub use in_place::deserialize_in_place;

/// Represents a fully-analyzed struct ready for code generation.
pub struct StructDef {
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub container_attrs: ContainerAttrs,
    pub fields: Vec<FieldDef>,
}

/// Represents a single field with all resolved attributes.
pub struct FieldDef {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub attrs: FieldAttrs,
}

/// Collects type parameter identifiers that appear in a type.
///
/// Given a type like `Vec<T>` and known type params `[T, U]`, this adds `T` to `found`.
/// Recurses into generic arguments, references, tuples, arrays, and slices.
pub(crate) fn collect_type_param_idents<'a>(ty: &Type, type_param_idents: &[&'a Ident], found: &mut Vec<&'a Ident>) {
    match ty {
        Type::Path(type_path) => {
            if type_path.qself.is_none() && type_path.path.segments.len() == 1 {
                let seg = &type_path.path.segments[0];
                if let Some(&ident) = type_param_idents.iter().find(|&&id| *id == seg.ident) {
                    found.push(ident);
                }
                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let GenericArgument::Type(inner_ty) = arg {
                            collect_type_param_idents(inner_ty, type_param_idents, found);
                        }
                    }
                }
            } else {
                for seg in &type_path.path.segments {
                    if let PathArguments::AngleBracketed(args) = &seg.arguments {
                        for arg in &args.args {
                            if let GenericArgument::Type(inner_ty) = arg {
                                collect_type_param_idents(inner_ty, type_param_idents, found);
                            }
                        }
                    }
                }
            }
        }
        Type::Reference(type_ref) => {
            collect_type_param_idents(&type_ref.elem, type_param_idents, found);
        }
        Type::Tuple(type_tuple) => {
            for elem in &type_tuple.elems {
                collect_type_param_idents(elem, type_param_idents, found);
            }
        }
        Type::Array(type_array) => {
            collect_type_param_idents(&type_array.elem, type_param_idents, found);
        }
        Type::Slice(type_slice) => {
            collect_type_param_idents(&type_slice.elem, type_param_idents, found);
        }
        _ => {}
    }
}
