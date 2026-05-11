use serde::ast::{Container, Style};
use serde::attr::{Default as SerdeDefault, RenameRule};
use serde::{Ctxt, Derive};
use serde_derive_internals as serde;
use syn::{parenthesized, parse_quote, token, Attribute, DeriveInput, Error, Expr, Ident, Member, Result, Token, Type};

/// Container-level serde attributes relevant to our derive.
pub struct ContainerAttrs {
    /// The `rename_all` rule applied to field names, if any.
    #[allow(dead_code)]
    pub rename_all: Option<RenameRule>,

    /// Whether the container has `#[serde(default)]`.
    pub has_default: bool,
}

/// Field-level serde attributes relevant to our derive.
pub struct FieldAttrs {
    /// The serialized name after applying `rename_all` and `rename`.
    pub serialized_name: String,

    /// Additional aliases from `#[serde(alias = "...")]`.
    pub aliases: Vec<String>,

    /// Whether the field has `#[serde(skip)]` or `#[serde(skip_deserializing)]`.
    pub skip_deserializing: bool,

    /// Whether the field has `#[serde(default)]`.
    pub has_field_default: bool,

    /// The default expression from `#[serde(default = "path")]`, if any.
    pub default_expr: Option<Expr>,
}

/// Parsed representation of a struct with all attribute information resolved.
pub struct ParsedStruct {
    pub container_attrs: ContainerAttrs,
    pub fields: Vec<ParsedField>,
}

/// A single field with its identity and resolved attributes.
pub struct ParsedField {
    pub ident: Ident,
    pub ty: Type,
    pub attrs: FieldAttrs,
}

/// The list of serde attributes that this derive macro does not support.
const UNSUPPORTED_ATTRS: &[&str] = &[
    "flatten",
    "with",
    "deserialize_with",
    "bound",
    "from",
    "try_from",
    "tag",
    "untagged",
];

/// Parse serde attributes from a `DeriveInput` representing a named struct.
///
/// # Remarks
///
/// This function uses `serde_derive_internals` to parse all serde attributes, then checks for unsupported attributes
/// and extracts the information we need.
///
/// Returns `Ok(ParsedStruct)` on success, or `Err(Error)` if unsupported attributes are detected or parsing fails.
pub fn parse_struct_attrs(input: &DeriveInput) -> Result<ParsedStruct> {
    // check for unsupported attributes before doing full parsing
    check_unsupported_attrs(input)?;

    // Use serde_derive_internals to parse all attributes.
    let cx = Ctxt::new();
    let container = match Container::from_ast(&cx, input, Derive::Deserialize) {
        Some(c) => c,
        None => {
            // If Container::from_ast returns None, check for errors from cx.
            return Err(cx.check().unwrap_err());
        }
    };

    // check if serde_derive_internals reported any errors
    cx.check()?;

    // extract container-level attributes
    let rename_all_rules = container.attrs.rename_all_rules();
    let rename_rule = rename_all_rules.deserialize;
    let rename_all = if rename_rule == RenameRule::None {
        None
    } else {
        Some(rename_rule)
    };
    let has_default = !container.attrs.default().is_none();
    let container_attrs = ContainerAttrs {
        rename_all,
        has_default,
    };

    // extract field-level attributes
    let fields = match &container.data {
        serde::ast::Data::Struct(Style::Struct, fields) => fields,
        _ => {
            return Err(Error::new_spanned(
                input,
                "config::Deserialize can only be derived for structs with named fields",
            ));
        }
    };

    let parsed_fields: Vec<ParsedField> = fields
        .iter()
        .map(|field| {
            let ident = match &field.member {
                Member::Named(ident) => ident.clone(),
                Member::Unnamed(_) => unreachable!("named struct fields have idents"),
            };

            let serialized_name = field.attrs.name().deserialize_name().to_owned();

            let aliases: Vec<String> = field
                .attrs
                .aliases()
                .iter()
                .filter(|a| *a != &serialized_name)
                .cloned()
                .collect();

            let skip_deserializing = field.attrs.skip_deserializing();

            let (has_field_default, default_expr) = match field.attrs.default() {
                SerdeDefault::None => (false, None),
                SerdeDefault::Default => (true, None),
                SerdeDefault::Path(path) => {
                    let expr: Expr = parse_quote!(#path());
                    (true, Some(expr))
                }
            };

            ParsedField {
                ident,
                ty: field.ty.clone(),
                attrs: FieldAttrs {
                    serialized_name,
                    aliases,
                    skip_deserializing,
                    has_field_default,
                    default_expr,
                },
            }
        })
        .collect();

    Ok(ParsedStruct {
        container_attrs,
        fields: parsed_fields,
    })
}

/// Check for unsupported serde attributes on the container and its fields.
///
/// Scans `#[serde(...)]` attributes and emits compile errors for any that
/// this derive macro does not support.
fn check_unsupported_attrs(input: &DeriveInput) -> Result<()> {
    let mut errors: Vec<Error> = Vec::new();

    // check container-level attributes
    for attr in &input.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        check_meta_list_for_unsupported(attr, &mut errors);
    }

    // check field-level attributes
    if let syn::Data::Struct(data) = &input.data {
        for field in &data.fields {
            for attr in &field.attrs {
                if !attr.path().is_ident("serde") {
                    continue;
                }
                check_meta_list_for_unsupported(attr, &mut errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let mut combined = errors.remove(0);
        for err in errors {
            combined.combine(err);
        }
        Err(combined)
    }
}

/// Parse a `#[serde(...)]` attribute's nested meta items and check for unsupported ones.
fn check_meta_list_for_unsupported(attr: &Attribute, errors: &mut Vec<Error>) {
    let _ = attr.parse_nested_meta(|meta| {
        for &unsupported in UNSUPPORTED_ATTRS {
            if meta.path.is_ident(unsupported) {
                errors.push(Error::new_spanned(
                    &meta.path,
                    format!("config::Deserialize does not support #[serde({})]", unsupported),
                ));
                // consume any value (= "...") or nested content so parsing continues
                if meta.input.peek(Token![=]) {
                    let _: Expr = meta.value()?.parse()?;
                } else if meta.input.peek(token::Paren) {
                    let _content;
                    parenthesized!(_content in meta.input);
                }
                return Ok(());
            }
        }

        // for supported attributes, consume their value so parsing continues
        if meta.input.peek(Token![=]) {
            let _: Expr = meta.value()?.parse()?;
        } else if meta.input.peek(token::Paren) {
            let _content;
            parenthesized!(_content in meta.input);
        }
        Ok(())
    });
}
