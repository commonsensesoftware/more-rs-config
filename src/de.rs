use crate::{Configuration, Section};
use serde::{
    de::{
        self,
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    Deserialize,
};
use std::{fmt::Display, iter::IntoIterator, rc::Rc, vec::IntoIter};
use thiserror::Error;

/// Represents the deserialization errors that can occur.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Indicates a value is missing for a field.
    #[error("Missing value for field '{0}'")]
    MissingValue(&'static str),

    /// Indicates a custom error message
    #[error("{0}")]
    Custom(String),
}

impl de::Error for Error {
    #[inline]
    fn custom<T: Display>(message: T) -> Self {
        Self::Custom(message.to_string())
    }

    #[inline]
    fn missing_field(field: &'static str) -> Self {
        Self::MissingValue(field)
    }
}

macro_rules! forward_parsed_values {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
                match self.0.value().parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(de::Error::custom(format_args!("{e} while parsing value '{}' provided by {}", self.0.value(), self.0.key())))
                }
            }
        )*
    }
}

// configuration is a key/value pair mapping of String: String or String: Vec<String>; however,
// we need a surrogate type to implement the deserialization on to underlying primitives
struct Key<'a>(Rc<Section<'a>>);

struct Val<'a>(Rc<Section<'a>>);

impl<'de> IntoDeserializer<'de, Error> for Key<'de> {
    type Deserializer = Self;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for Key<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.key().to_owned().into_deserializer().deserialize_any(visitor)
    }

    #[inline]
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_str(self.0.key())
    }

    #[inline]
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        char str string unit seq option
        bytes byte_buf map unit_struct tuple_struct
        tuple ignored_any enum
        struct bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64
    }
}

impl<'de> IntoDeserializer<'de, Error> for Val<'de> {
    type Deserializer = Self;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for Val<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.value().into_deserializer().deserialize_any(visitor)
    }

    // parse each numeric key exactly once, then sort by index. this is required to ensure the zero-based ordering
    // of the sequence entries (e.g. array) are retained.
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let mut indexed: Vec<_> = self
            .0
            .sections()
            .into_iter()
            .filter_map(|s| s.key().parse::<usize>().ok().map(|i| (i, s)))
            .collect();

        indexed.sort_by_key(|(i, _)| *i);

        let values = indexed.into_iter().map(|(_, s)| Val(Rc::new(s)));

        SeqDeserializer::new(values).deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let values = self.0.sections().into_iter().map(|section| {
            let section = Rc::new(section);
            (Key(Rc::clone(&section)), Val(section))
        });

        MapDeserializer::new(values).deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_some(self)
    }

    forward_parsed_values! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }

    #[inline]
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        Deserializer::from_ref(self.0).deserialize_struct(name, fields, visitor)
    }

    #[inline]
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let value = self.0.value();

        if !value.is_empty() {
            // unit variant: the value itself is the variant name (e.g. "First")
            visitor.visit_enum(value.into_deserializer())
        } else {
            // non-scalar variant: a subsection key is the variant name
            // and its value/children are the variant data (e.g. Second: "test")
            let sections = self.0.sections();

            if let Some(section) = sections.into_iter().next() {
                visitor.visit_enum(EnumDeserializer(section))
            } else {
                visitor.visit_enum(value.into_deserializer())
            }
        }
    }

    serde::forward_to_deserialize_any! {
        char str string unit
        bytes byte_buf unit_struct tuple_struct
        identifier tuple ignored_any
    }
}

struct ConfigValues<'a>(IntoIter<Section<'a>>);

struct EnumDeserializer<'a>(Section<'a>);

impl<'de> de::EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant = self.0.key().to_owned();
        let val = seed.deserialize(variant.into_deserializer())?;
        Ok((val, self))
    }
}

impl<'de> de::VariantAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Self::Error> {
        seed.deserialize(Val(Rc::new(self.0)))
    }

    #[inline]
    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error> {
        de::Deserializer::deserialize_seq(Val(Rc::new(self.0)), visitor)
    }

    #[inline]
    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        de::Deserializer::deserialize_struct(Deserializer(self.0.sections().into_iter()), "", fields, visitor)
    }
}

impl<'a> Iterator for ConfigValues<'a> {
    type Item = (Key<'a>, Val<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|section| {
            let section = Rc::new(section);
            (Key(Rc::clone(&section)), Val(section))
        })
    }
}

struct Deserializer<'de>(IntoIter<Section<'de>>);

fn fields_match(config_key: &str, field: &str) -> bool {
    // compare characters case-insensitively, skipping underscores in the field name. this allows PascalCase config
    // keys (ex: "MagicNumbers") to match snake_case Rust fields (ex: "magic_numbers")
    let mut key_chars = config_key.chars();
    let mut field_chars = field.chars().filter(|&c| c != '_');

    loop {
        match (key_chars.next(), field_chars.next()) {
            (Some(a), Some(b)) if a.eq_ignore_ascii_case(&b) => continue,
            (None, None) => return true,
            _ => return false,
        }
    }
}

struct FieldMappingAccess<'de> {
    sections: IntoIter<Section<'de>>,
    fields: &'static [&'static str],
    pending_value: Option<Rc<Section<'de>>>,
}

impl<'de> de::MapAccess<'de> for FieldMappingAccess<'de> {
    type Error = Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> {
        while let Some(section) = self.sections.next() {
            let config_key = section.key();

            if let Some(&field) = self.fields.iter().find(|f| fields_match(config_key, f)) {
                self.pending_value = Some(Rc::new(section));
                return seed.deserialize(field.into_deserializer()).map(Some);
            }
        }

        Ok(None)
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
        let section = self.pending_value.take().expect("next_value_seed called before next_key_seed");
        seed.deserialize(Val(section))
    }
}

impl<'de> From<&'de Configuration> for Deserializer<'de> {
    #[inline]
    fn from(config: &'de Configuration) -> Self {
        Self(config.sections().into_iter())
    }
}

impl<'de> Deserializer<'de> {
    fn from_ref(section: Rc<Section<'de>>) -> Self {
        match Rc::try_unwrap(section) {
            Ok(section) => Self::from(section),
            Err(section) => Self((*section).sections().into_iter()),
        }
    }
}

impl<'de> From<Section<'de>> for Deserializer<'de> {
    #[inline]
    fn from(section: Section<'de>) -> Self {
        Self(section.sections().into_iter())
    }
}

impl<'de> From<Vec<Section<'de>>> for Deserializer<'de> {
    #[inline]
    fn from(sections: Vec<Section<'de>>) -> Self {
        Self(sections.into_iter())
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }

    #[inline]
    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(MapDeserializer::new(ConfigValues(self.0)))
    }

    #[inline]
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(FieldMappingAccess {
            sections: self.0,
            fields,
            pending_value: None,
        })
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf unit_struct tuple_struct
        identifier tuple ignored_any option newtype_struct enum
    }
}

/// Deserializes a data structure from the specified configuration sections.
///
/// # Arguments
///
/// * `configuration` - The configuration [sections](Section) to deserialize
#[inline]
pub fn from<'a, T: Deserialize<'a>>(configuration: impl Into<Vec<Section<'a>>>) -> Result<T, Error> {
    T::deserialize(Deserializer::from(configuration.into()))
}

/// Deserializes the specified configuration to an existing data structure.
///
/// # Arguments
///
/// * `configuration` - The configuration [sections](Section) to bind to the data
/// * `data` - The data to bind the configuration to
#[inline]
pub fn bind<'a, T: Deserialize<'a>>(sections: impl Into<Vec<Section<'a>>>, data: &mut T) -> Result<(), Error> {
    T::deserialize_in_place(Deserializer::from(sections.into()), data)
}
