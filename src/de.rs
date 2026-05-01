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
        identifier tuple ignored_any enum
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
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        de::Deserializer::deserialize_any(Deserializer::from_ref(self.0), visitor)
    }

    #[inline]
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self.0.value().into_deserializer())
    }

    serde::forward_to_deserialize_any! {
        char str string unit
        bytes byte_buf unit_struct tuple_struct
        identifier tuple ignored_any
    }
}

struct ConfigValues<'a>(IntoIter<Section<'a>>);

impl<'a> Iterator for ConfigValues<'a> {
    type Item = (Key<'a>, Val<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|section| {
            let section = Rc::new(section);
            (Key(Rc::clone(&section)), Val(section))
        })
    }
}

struct Deserializer<'de>(MapDeserializer<'de, ConfigValues<'de>, Error>);

impl<'de> From<&'de Configuration> for Deserializer<'de> {
    #[inline]
    fn from(config: &'de Configuration) -> Self {
        Self(MapDeserializer::new(ConfigValues(config.sections().into_iter())))
    }
}

impl<'de> Deserializer<'de> {
    fn from_ref(section: Rc<Section<'de>>) -> Self {
        match Rc::try_unwrap(section) {
            Ok(section) => Self::from(section),
            Err(section) => Self(MapDeserializer::new(ConfigValues((*section).sections().into_iter()))),
        }
    }
}

impl<'de> From<Section<'de>> for Deserializer<'de> {
    #[inline]
    fn from(section: Section<'de>) -> Self {
        Self(MapDeserializer::new(ConfigValues(section.sections().into_iter())))
    }
}

impl<'de> From<Vec<Section<'de>>> for Deserializer<'de> {
    #[inline]
    fn from(sections: Vec<Section<'de>>) -> Self {
        Self(MapDeserializer::new(ConfigValues(sections.into_iter())))
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
        visitor.visit_map(self.0)
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf unit_struct tuple_struct
        identifier tuple ignored_any option newtype_struct enum
        struct
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
