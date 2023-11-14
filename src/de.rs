use crate::{Configuration, ConfigurationSection};
use serde::{
    de::{
        self,
        value::{MapDeserializer, SeqDeserializer},
        IntoDeserializer, Visitor,
    },
    Deserialize,
};
use std::{
    fmt::{self, Display, Formatter},
    iter::IntoIterator,
    vec::IntoIter,
};

/// Represents the deserialization errors that can occur.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Indicates a value is missing
    MissingValue(&'static str),

    /// Indicates a custom error message
    Custom(String),
}

impl de::Error for Error {
    fn custom<T: Display>(message: T) -> Self {
        Error::Custom(message.to_string())
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingValue(field)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Error::MissingValue(field) => {
                formatter.write_str("missing value for field ")?;
                formatter.write_str(field)
            }
            Error::Custom(ref msg) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MissingValue(_) => "missing value",
            Error::Custom(_) => "custom error",
        }
    }
}

macro_rules! forward_parsed_values {
    ($($ty:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where V: de::Visitor<'de>
            {
                match self.0.value().parse::<$ty>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(de::Error::custom(format_args!("{} while parsing value '{}' provided by {}", e, self.0.value(), self.0.key())))
                }
            }
        )*
    }
}

// configuration is a key/value pair mapping of String: String or String: Vec<String>; however,
// we need a surrogate type to implement forward the deserialization on to underlying primitives
struct Key(String);
struct Value(Box<dyn ConfigurationSection>);

impl<'de> IntoDeserializer<'de, Error> for Key {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for Key {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.into_deserializer().deserialize_any(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        char str string unit seq option
        bytes byte_buf map unit_struct tuple_struct
        identifier tuple ignored_any enum
        struct bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64
    }
}

impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.value().into_owned().into_deserializer().deserialize_any(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let mut values: Vec<_> = self
            .0
            .children()
            .into_iter()
            .take_while(|c| c.key().parse::<usize>().is_ok())
            .map(|s| Value(s))
            .collect();

        // guarantee stable ordering by zero-based ordinal index; for example,
        // Key:0
        // Key:1
        // Key:n
        values.sort_by(|s1, s2| {
            s1.0.key()
                .parse::<usize>()
                .unwrap()
                .cmp(&s2.0.key().parse::<usize>().unwrap())
        });

        SeqDeserializer::new(values.into_iter()).deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
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

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let config = self.0.deref();
        let deserializer = Deserializer::new(config);
        de::Deserializer::deserialize_any(deserializer, visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self.0.value().into_owned().into_deserializer())
    }

    serde::forward_to_deserialize_any! {
        char str string unit
        bytes byte_buf map unit_struct tuple_struct
        identifier tuple ignored_any
    }
}

struct ConfigValues(IntoIter<Box<dyn ConfigurationSection>>);

impl Iterator for ConfigValues {
    type Item = (Key, Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|section| (Key(section.key().to_owned()), Value(section)))
    }
}

struct Deserializer<'de> {
    inner: MapDeserializer<'de, ConfigValues, Error>,
}

impl<'de> Deserializer<'de> {
    fn new(config: &dyn Configuration) -> Self {
        Deserializer {
            inner: MapDeserializer::new(ConfigValues(config.children().into_iter())),
        }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.inner)
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf unit_struct tuple_struct
        identifier tuple ignored_any option newtype_struct enum
        struct
    }
}

/// Deserializes a data structure from the specified configuration.
///
/// # Arguments
///
/// * `configuration` - The [configuration](trait.Configuration.html) to deserialize
pub fn from_config<'a, T>(configuration: &'a dyn Configuration) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    Ok(T::deserialize(Deserializer::new(configuration))?)
}

/// Deserializes the specified configuration to an existing data structure.
///
/// # Arguments
///
/// * `configuration` - The [configuration](trait.Configuration.html) to deserialize
pub fn bind_config<'a, T>(configuration: &'a dyn Configuration, data: &mut T) -> Result<(), Error>
where
    T: Deserialize<'a>,
{
    Ok(T::deserialize_in_place(Deserializer::new(configuration), data)?)
}