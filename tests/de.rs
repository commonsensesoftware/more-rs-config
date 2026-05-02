use config::{prelude::*, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    Small,

    #[default]
    Medium,
    Large,
}

pub fn default_kaboom() -> u16 {
    8080
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CustomNewType(u32);

#[derive(Deserialize, Debug, PartialEq)]
pub struct Foo {
    bar: String,
    baz: bool,
    zoom: Option<u16>,
    doom: Vec<u64>,
    boom: Vec<String>,
    #[serde(default = "default_kaboom")]
    kaboom: u16,
    #[serde(default)]
    debug_mode: bool,
    #[serde(default)]
    size: Size,
    provided: Option<String>,
    new_type: CustomNewType,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Name {
    first: String,
    last: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct GrandChild {
    name: Name,
    age: u8,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Child {
    name: Name,
    magic_numbers: Vec<u8>,
    children: Vec<GrandChild>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Parent {
    name: Name,
    child: Child,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct UserDefined {
    settings: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Place {
    pub name: String,
    pub enums: Vec<Enum>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum Enum {
    First,
    Second(String),

    Third {
        id: usize,
        kind: String,
    },
}

#[test]
fn from_should_deserialize_simple_struct() -> Result {
    let config = config::builder()
        .add_in_memory(&[
            ("Bar", "test"),
            ("Baz", "true"),
            ("Doom:0", "1"),
            ("Doom:1", "2"),
            ("Doom:2", "3"),
            ("Boom", ""),
            ("Size", "small"),
            ("Provided", "test"),
            ("NewType", "42"),
            ("Child:Ignored", "42"),
        ])
        .build()?;
    let expected = Foo {
        bar: String::from("test"),
        baz: true,
        zoom: None,
        doom: vec![1, 2, 3],
        boom: vec![],
        kaboom: 8080,
        debug_mode: false,
        size: Size::Small,
        provided: Some(String::from("test")),
        new_type: CustomNewType(42),
    };

    // act
    let actual = config::de::from::<Foo>(&config)?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn from_should_deserialize_parent_child_struct() -> Result {
    let config = config::builder()
        .add_in_memory(&[
            ("Name:First", "Jane"),
            ("Name:Last", "Doe"),
            ("Child:Name:First", "John"),
            ("Child:Name:Last", "Doe"),
            ("Child:MagicNumbers:0", "42"),
            ("Child:MagicNumbers:1", "7"),
            ("Child:MagicNumbers:2", "13"),
            ("Child:Children:0:Name:First", "Bob"),
            ("Child:Children:0:Name:Last", "Doe"),
            ("Child:Children:0:Age", "7"),
            ("Child:Children:1:Name:First", "Sally"),
            ("Child:Children:1:Name:Last", "Doe"),
            ("Child:Children:1:Age", "5"),
        ])
        .build()?;
    let expected = Parent {
        name: Name {
            first: String::from("Jane"),
            last: String::from("Doe"),
        },
        child: Child {
            name: Name {
                first: String::from("John"),
                last: String::from("Doe"),
            },
            magic_numbers: vec![42, 7, 13],
            children: vec![
                GrandChild {
                    name: Name {
                        first: String::from("Bob"),
                        last: String::from("Doe"),
                    },
                    age: 7,
                },
                GrandChild {
                    name: Name {
                        first: String::from("Sally"),
                        last: String::from("Doe"),
                    },
                    age: 5,
                },
            ],
        },
    };

    // act
    let actual = config::de::from::<Parent>(&config)?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn from_should_fail_with_missing_value() -> Result {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Bar", "test"), ("Baz", "true")])
        .build()?;

    // act
    let error = config::de::from::<Foo>(&config).err().unwrap();

    // assert
    assert_eq!(error, config::de::Error::MissingValue("doom"));
    Ok(())
}

#[test]
fn from_should_fail_with_invalid_type() -> Result {
    // arrange
    let config = config::builder()
        .add_in_memory(&[
            ("Bar", "test"),
            ("Baz", "notabool"),
            ("Doom:0", "1"),
            ("Doom:1", "2"),
            ("Doom:2", "3"),
        ])
        .build()?;

    // act
    let error = config::de::from::<Foo>(&config).err().unwrap();

    // assert
    assert_eq!(
        error,
        config::de::Error::Custom(String::from(
            "provided string was not `true` or `false` while parsing value \'notabool\' provided by Baz"
        ))
    );
    Ok(())
}

#[test]
fn from_should_deserialize_nested_string_map() -> Result {
    // arrange
    let config = config::builder()
        .add_in_memory(&[
            ("Dimensions:Foo", "bar"),
            ("Dimensions:Bar", "foo"),
            ("Dimensions:Baz", "other"),
        ])
        .build()?;
    let expected = [("Foo", "bar"), ("Bar", "foo"), ("Baz", "other")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>();

    // act
    let actual = config::de::from::<HashMap<String, String>>(config.section("Dimensions"))?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn from_should_deserialize_nested_typed_map() -> Result {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Limits:Foo", "42"), ("Limits:Bar", "0"), ("Limits:Baz", "420")])
        .build()?;
    let expected = [("Foo", 42), ("Bar", 0), ("Baz", 420)]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect::<HashMap<_, usize>>();

    // act
    let actual = config::de::from::<HashMap<String, usize>>(config.section("Limits"))?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn from_should_deserialize_map_with_nested_vec() -> Result {
    // arrange
    let config = config::builder()
        .add_in_memory(&[
            ("Settings:Key1:0", "bar"),
            ("Settings:Key2:0", "foo"),
            ("Settings:Key3:0", "a"),
            ("Settings:Key3:1", "b"),
            ("Settings:Key3:2", "c"),
        ])
        .build()?;
    let expected = HashMap::from([
        ("Key1".to_owned(), vec!["bar".to_owned()]),
        ("Key2".to_owned(), vec!["foo".to_owned()]),
        ("Key3".to_owned(), vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]),
    ]);

    // act
    let actual = config::de::from::<HashMap<String, Vec<String>>>(config.section("Settings"))?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn reify_should_deserialize_map_with_nested_vec() -> Result {
    // arrange
    let config = config::builder()
        .add_in_memory(&[
            ("Settings:Key1:0", "bar"),
            ("Settings:Key2:0", "foo"),
            ("Settings:Key3:0", "a"),
            ("Settings:Key3:1", "b"),
            ("Settings:Key3:2", "c"),
        ])
        .build()?;
    let expected = UserDefined {
        settings: HashMap::from([
            ("Key1".to_owned(), vec!["bar".to_owned()]),
            ("Key2".to_owned(), vec!["foo".to_owned()]),
            ("Key3".to_owned(), vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]),
        ]),
    };

    // act
    let actual: UserDefined = config.reify()?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn from_should_deserialize_deep_nested_map() -> Result {
    // arrange
    let config = config::builder()
        .add_in_memory(&[
            ("Key1:0:Subkey1_0:0", "1"),
            ("Key1:0:Subkey1_0:1", "2"),
            ("Key1:0:Subkey1_0:2", "3"),
            ("Key1:0:Subkey2_0:0", "4"),
            ("Key1:0:Subkey2_0:1", "5"),
            ("Key1:0:Subkey2_0:2", "6"),
            ("Key1:1:Subkey3_0:0", "7"),
            ("Key1:1:Subkey3_0:1", "8"),
            ("Key1:1:Subkey3_0:2", "9"),
            ("Key2:0:Subkey4_0:0", "a"),
            ("Key2:0:Subkey4_0:1", "b"),
            ("Key2:0:Subkey4_0:2", "c"),
            ("Key3:0:Subkey5_0:0", "d"),
            ("Key3:0:Subkey5_0:1", "e"),
            ("Key3:0:Subkey5_0:2", "f"),
            ("Key3:0:Subkey6_0:0", "x"),
            ("Key3:0:Subkey6_0:1", "y"),
            ("Key3:0:Subkey6_0:2", "z"),
        ])
        .build()?;
    let expected = HashMap::from([
        (
            "Key1".to_owned(),
            vec![
                HashMap::from([
                    (
                        "Subkey1_0".to_owned(),
                        vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
                    ),
                    (
                        "Subkey2_0".to_owned(),
                        vec!["4".to_owned(), "5".to_owned(), "6".to_owned()],
                    ),
                ]),
                HashMap::from([(
                    "Subkey3_0".to_owned(),
                    vec!["7".to_owned(), "8".to_owned(), "9".to_owned()],
                )]),
            ],
        ),
        (
            "Key2".to_owned(),
            vec![HashMap::from([(
                "Subkey4_0".to_owned(),
                vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
            )])],
        ),
        (
            "Key3".to_owned(),
            vec![HashMap::from([
                (
                    "Subkey5_0".to_owned(),
                    vec!["d".to_owned(), "e".to_owned(), "f".to_owned()],
                ),
                (
                    "Subkey6_0".to_owned(),
                    vec!["x".to_owned(), "y".to_owned(), "z".to_owned()],
                ),
            ])],
        ),
    ]);

    // act
    let actual = config::de::from::<HashMap<String, Vec<HashMap<String, Vec<String>>>>>(&config)?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn reify_should_deserialize_non_scalar_enum() -> Result {
    // arrange
    let config = config::builder()
        .add_in_memory(&[
            ("Name", "some name"),
            ("Enums:0", "First"),
            ("Enums:1:Second", "test"),
            ("Enums:2:Third:Id", "42"),
            ("Enums:2:Third:Kind", "Encounter"),
        ])
        .build()?;
    let expected = Place {
        name: "some name".into(),
        enums: vec![
            Enum::First,
            Enum::Second("test".into()),
            Enum::Third {
                id: 42,
                kind: "Encounter".into(),
            },
        ],
    };

    // act
    let actual: Place = config.reify()?;

    // assert
    assert_eq!(actual, expected);
    Ok(())
}
