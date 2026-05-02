use super::Error;
use crate::{pascal_case, path, Settings};
use serde::ser::{
    self, Impossible, SerializeMap, SerializeSeq, SerializeStructVariant, SerializeTuple, SerializeTupleStruct,
    SerializeTupleVariant,
};
use serde::Serialize;

struct Serializer<'a> {
    paths: Vec<String>,
    settings: &'a mut Settings,
}

impl<'a> Serializer<'a> {
    #[inline]
    fn new(settings: &'a mut Settings) -> Self {
        Self {
            paths: Vec::new(),
            settings,
        }
    }

    fn enter_context(&mut self, segment: &str) {
        let new_path = if let Some(current) = self.paths.last() {
            path::combine(&[current, segment])
        } else {
            segment.to_owned()
        };
        self.paths.push(new_path);
    }

    #[inline]
    fn exit_context(&mut self) {
        self.paths.pop();
    }

    fn add_value(&mut self, value: impl ToString) {
        let current = self.paths.last().map(|s| s.as_str()).unwrap_or("");
        self.settings.insert(pascal_case(current), value.to_string());
    }
}

struct SerializeComplex<'a, 'b> {
    ser: &'a mut Serializer<'b>,
    key: Option<String>,
    pop_on_end: bool,
}

impl<'a, 'b> ser::SerializeStruct for SerializeComplex<'a, 'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
        self.ser.enter_context(&pascal_case(key));
        value.serialize(&mut *self.ser)?;
        self.ser.exit_context();
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'b> SerializeStructVariant for SerializeComplex<'a, 'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error> {
        self.ser.enter_context(&pascal_case(key));
        value.serialize(&mut *self.ser)?;
        self.ser.exit_context();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.pop_on_end {
            self.ser.exit_context();
        }
        Ok(())
    }
}

impl<'a, 'b> SerializeMap for SerializeComplex<'a, 'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + serde::Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let key_string = key.serialize(MapKeySerializer)?;
        self.ser.enter_context(&key_string);
        self.key = Some(key_string);
        Ok(())
    }

    fn serialize_value<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.ser)?;
        self.ser.exit_context();
        self.key = None;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct SerializeComplexSeq<'a, 'b> {
    ser: &'a mut Serializer<'b>,
    index: usize,
    pop_on_end: bool,
}

impl<'a, 'b> SerializeSeq for SerializeComplexSeq<'a, 'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.ser.enter_context(&self.index.to_string());
        value.serialize(&mut *self.ser)?;
        self.ser.exit_context();
        self.index += 1;
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'b> SerializeTuple for SerializeComplexSeq<'a, 'b> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'b> SerializeTupleStruct for SerializeComplexSeq<'a, 'b> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'b> SerializeTupleVariant for SerializeComplexSeq<'a, 'b> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.pop_on_end {
            self.ser.exit_context();
        }
        Ok(())
    }
}

/// A minimal serializer that converts map keys to strings.
///
/// Accepts strings, integers, bools, and chars (via `ToString`).
/// Returns [`Error::NonStringKey`] for all other types.
struct MapKeySerializer;

impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<String, Error> {
        Ok(v.to_string())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<String, Error> {
        Ok(v.to_owned())
    }

    #[inline]
    fn serialize_bytes(self, _v: &[u8]) -> Result<String, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_none(self) -> Result<String, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_some<T: ?Sized + serde::Serialize>(self, _value: &T) -> Result<String, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_unit(self) -> Result<String, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String, Error> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &'static str, _value: &T) -> Result<String, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Error> {
        Err(Error::NonStringKey)
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(Error::NonStringKey)
    }
}

impl<'a, 'b> ser::Serializer for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeComplexSeq<'a, 'b>;
    type SerializeTuple = SerializeComplexSeq<'a, 'b>;
    type SerializeTupleStruct = SerializeComplexSeq<'a, 'b>;
    type SerializeTupleVariant = SerializeComplexSeq<'a, 'b>;
    type SerializeMap = SerializeComplex<'a, 'b>;
    type SerializeStruct = SerializeComplex<'a, 'b>;
    type SerializeStructVariant = SerializeComplex<'a, 'b>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::Custom("byte arrays are not supported".into()))
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.add_value("");
        Ok(())
    }

    #[inline]
    fn serialize_some<T: ?Sized + serde::Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.add_value("");
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.add_value("");
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.add_value(variant);
        Ok(())
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.enter_context(variant);
        value.serialize(&mut *self)?;
        self.exit_context();
        Ok(())
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeComplexSeq {
            ser: self,
            index: 0,
            pop_on_end: false,
        })
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeComplexSeq {
            ser: self,
            index: 0,
            pop_on_end: false,
        })
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeComplexSeq {
            ser: self,
            index: 0,
            pop_on_end: false,
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.enter_context(variant);
        Ok(SerializeComplexSeq {
            ser: self,
            index: 0,
            pop_on_end: true,
        })
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeComplex {
            ser: self,
            key: None,
            pop_on_end: false,
        })
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeComplex {
            ser: self,
            key: None,
            pop_on_end: false,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.enter_context(variant);
        Ok(SerializeComplex {
            ser: self,
            key: None,
            pop_on_end: true,
        })
    }
}

/// Serializes a value into the given [settings](Settings), flattening it into colon-delimited, Pascal Case set of
/// key/value pairs.
///
/// # Remarks
///
/// This is the public entry point for struct serialization — the reciprocal of [crate::de::from].
pub fn into<T: Serialize>(value: &T, settings: &mut Settings) -> Result<(), Error> {
    value.serialize(&mut Serializer::new(settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Settings;
    use serde::Serialize;

    fn serialize_leaf_at<T: Serialize>(path: &str, value: &T) -> Settings {
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        serializer.enter_context(path);
        serde::Serialize::serialize(value, &mut serializer).unwrap();
        settings
    }

    #[test]
    fn new_should_create_serializer_with_empty_paths() {
        // arrange
        let mut settings = Settings::new();

        // act
        let serializer = Serializer::new(&mut settings);

        // assert
        assert!(serializer.paths.is_empty());
    }

    #[test]
    fn enter_context_should_pushe_segment_when_paths_empty() {
        // arrange
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        // act
        serializer.enter_context("parent");

        // assert
        assert_eq!(serializer.paths, vec!["parent"]);
    }

    #[test]
    fn enter_context_should_combine_with_current_path() {
        // arrange
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        // act
        serializer.enter_context("parent");
        serializer.enter_context("child");

        // assert
        assert_eq!(serializer.paths, vec!["parent", "parent:child"]);
    }

    #[test]
    fn enter_context_should_supports_three_levels() {
        // arrange
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        // act
        serializer.enter_context("a");
        serializer.enter_context("b");
        serializer.enter_context("c");

        // assert
        assert_eq!(serializer.paths, vec!["a", "a:b", "a:b:c"]);
    }

    #[test]
    fn exit_context_should_pop_last_path() {
        // arrange
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        serializer.enter_context("parent");
        serializer.enter_context("child");

        // act
        serializer.exit_context();

        // assert
        assert_eq!(serializer.paths, vec!["parent"]);
    }

    #[test]
    fn exit_context_on_empty_paths_should_do_nothing() {
        // arrange
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        // act
        serializer.exit_context();

        // assert
        assert!(serializer.paths.is_empty());
    }

    #[test]
    fn add_value_should_insert_with_pascal_case_key() {
        // arrange
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        serializer.enter_context("hello_world");

        // act
        serializer.add_value("test");

        // assert
        assert_eq!(settings.get("HelloWorld"), Some("test"));
    }

    #[test]
    fn add_value_with_nested_path_should_use_pascal_case() {
        // arrange
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        serializer.enter_context("parent");
        serializer.enter_context("child_name");

        // act
        serializer.add_value(42);

        // assert
        assert_eq!(settings.get("Parent:ChildName"), Some("42"));
    }

    #[test]
    fn add_value_with_empty_paths_should_use_empty_key() {
        // arrange
        let mut settings = Settings::new();
        let mut serializer = Serializer::new(&mut settings);

        // act
        serializer.add_value("root_value");

        // assert
        assert_eq!(settings.get(""), Some("root_value"));
    }

    #[test]
    fn serialize_bool_should_write_true() {
        // arrange

        // act
        let settings = serialize_leaf_at("flag", &true);

        // assert
        assert_eq!(settings.get("Flag"), Some("true"));
    }

    #[test]
    fn serialize_bool_should_write_false() {
        // arrange

        // act
        let settings = serialize_leaf_at("flag", &false);

        // assert
        assert_eq!(settings.get("Flag"), Some("false"));
    }

    #[test]
    fn serialize_should_write_i8() {
        // arrange

        // act
        let settings = serialize_leaf_at("val", &(-42i8));

        // assert
        assert_eq!(settings.get("Val"), Some("-42"));
    }

    #[test]
    fn serialize_should_write_i64() {
        // arrange

        // act
        let settings = serialize_leaf_at("val", &(123456789i64));

        // assert
        assert_eq!(settings.get("Val"), Some("123456789"));
    }

    #[test]
    fn serialize_should_write_u8() {
        // arrange

        // act
        let settings = serialize_leaf_at("val", &(255u8));

        // assert
        assert_eq!(settings.get("Val"), Some("255"));
    }

    #[test]
    fn serialize_should_write_u64() {
        // arrange

        // act
        let settings = serialize_leaf_at("val", &(u64::MAX));

        // assert
        assert_eq!(settings.get("Val"), Some("18446744073709551615"));
    }

    #[test]
    fn serialize_should_write_f32() {
        // arrange

        // act
        let settings = serialize_leaf_at("val", &(3.14f32));

        // assert
        assert_eq!(settings.get("Val"), Some(3.14f32.to_string().as_str()));
    }

    #[test]
    fn serialize_should_write_f64() {
        // arrange

        // act
        let settings = serialize_leaf_at("val", &(2.718f64));

        // assert
        assert_eq!(settings.get("Val"), Some("2.718"));
    }

    #[test]
    fn serialize_should_write_char() {
        // arrange

        // act
        let settings = serialize_leaf_at("sep", &',');

        // assert
        assert_eq!(settings.get("Sep"), Some(","));
    }

    #[test]
    fn serialize_should_write_string() {
        // arrange

        // act
        let settings = serialize_leaf_at("name", &"Alice");

        // assert
        assert_eq!(settings.get("Name"), Some("Alice"));
    }

    #[test]
    fn serialize_should_write_unit_as_empty_string() {
        // arrange

        // act
        let settings = serialize_leaf_at("marker", &());

        // assert
        assert_eq!(settings.get("Marker"), Some(""));
    }

    #[test]
    fn serialize_should_write_none_as_empty_string() {
        // arrange

        // act
        let settings = serialize_leaf_at("opt", &Option::<i32>::None);

        // assert
        assert_eq!(settings.get("Opt"), Some(""));
    }

    #[test]
    fn serialize_some_should_be_transparent() {
        // arrange

        // act
        let settings = serialize_leaf_at("opt", &Some(42i32));

        // assert
        assert_eq!(settings.get("Opt"), Some("42"));
    }

    #[test]
    fn serialize_should_write_enum_variant() {
        // arrange
        #[derive(Serialize)]
        enum Size {
            Small,
        }

        // act
        let settings = serialize_leaf_at("size", &Size::Small);

        // assert
        assert_eq!(settings.get("Size"), Some("Small"));
    }

    #[test]
    fn serialize_should_write_transparent_struct() {
        // arrange
        #[derive(Serialize)]
        struct Wrapper(i32);

        // act
        let settings = serialize_leaf_at("val", &Wrapper(42));

        // assert
        assert_eq!(settings.get("Val"), Some("42"));
    }
}
