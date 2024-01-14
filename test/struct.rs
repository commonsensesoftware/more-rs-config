use config::{ext::*, *};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[test]
#[should_panic]
fn load_unit_struct() {
    // Serializing empty struct is unsupported
    #[derive(Deserialize, Serialize, Clone)]
    #[serde(rename_all(serialize = "PascalCase"))]
    #[serde(rename_all(deserialize = "PascalCase"))]
    struct A;
    let aa: A = A{};

    let _ = DefaultConfigurationBuilder::new()
        .add_struct(aa)
        .build()
        .unwrap();
}

#[test]
fn load_simple_struct() {
    #[derive(Deserialize, Serialize, Clone)]
    #[serde(rename_all(serialize = "PascalCase"))]
    #[serde(rename_all(deserialize = "PascalCase"))]
    struct A {
        a: i32,
        b: bool,
    }
    let aa = A {
        a: 32,
        b: false
    };

    let config = DefaultConfigurationBuilder::new()
        .add_struct(aa.clone())
        .build()
        .unwrap();
    assert_eq!(config.get("a").unwrap().as_str(), "32");
    assert_eq!(config.get("b").unwrap().as_str(), "false");
    let bb: A = config.reify();
    assert_eq!(aa.a, bb.a);
    assert_eq!(aa.b, bb.b);
}

#[test]
#[should_panic]
fn load_bool() {
    // Serializing primitive types is not supported
    for value in [true, false] {
        let config = DefaultConfigurationBuilder::new()
            .add_struct(value)
            .build()
            .unwrap();
        let loaded = config.get("");
        assert_eq!(loaded.unwrap().as_str(), value.to_string().as_str());
    }
}

#[test]
fn load_map_str_str() {
    let mut value: HashMap<&str, &str> = HashMap::new();
    value.insert("foo", "bar");
    value.insert("karoucho", "34");

    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    let loaded = config.get("foo");
    assert_eq!(loaded.unwrap().as_str(), "bar");

    #[derive(Deserialize)]
    #[serde(rename_all(deserialize = "PascalCase"))]
    struct FooBar {
        foo: String,
        karoucho: i32,
    }
    let options: FooBar = config.reify();
    assert_eq!(options.foo, "bar".to_string());
    assert_eq!(options.karoucho, 34);
}

#[test]
fn load_map_bool_i32() {
    let mut value: HashMap<bool, i32> = HashMap::new();
    value.insert(false, 56);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    let loaded = config.get("false");
    assert_eq!(loaded.unwrap().as_str(), "56");

    #[derive(Deserialize, Serialize, Clone)]
    #[serde(rename_all(serialize = "PascalCase"))]
    #[serde(rename_all(deserialize = "PascalCase"))]
    struct BoolI32 {
        r#false: i32
    }

    let my: BoolI32 = config.reify();
    assert_eq!(my.r#false, 56);
}

#[test]
fn load_map_i32_i32() {
    let mut value: HashMap<i32, i32> = HashMap::new();
    value.insert(-32, 56);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    let loaded = config.get("-32_i32");
    assert_eq!(loaded.unwrap().as_str(), "56");
    // Cannot reify into struct with "-32" as member name
}

#[test]
fn load_tuple_bool_i32() {
    let value = (false, 56);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    let value0 = config.get("0");
    assert_eq!(value0.unwrap().as_str(), "false");
    let value1 = config.get("1");
    assert_eq!(value1.unwrap().as_str(), "56");
}

#[test]
fn load_vec_i32() {
    let value = std::vec::Vec::from([32, 56]);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    let value0 = config.get("0");
    assert_eq!(value0.unwrap().as_str(), "32");
    let value1 = config.get("1");
    assert_eq!(value1.unwrap().as_str(), "56");
}

#[test]
fn load_vec_replaces_and_appends() {
    let vec1 = std::vec::Vec::from([32, 56]);
    let vec2 = std::vec::Vec::from([96, 3, 54]);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(vec1)
        .add_struct(vec2)
        .build()
        .unwrap();
    let value0 = config.get("0");
    assert_eq!(value0.unwrap().as_str(), "96");
    let value1 = config.get("1");
    assert_eq!(value1.unwrap().as_str(), "3");
    let value1 = config.get("2");
    assert_eq!(value1.unwrap().as_str(), "54");
}

#[test]
fn load_explicit_vec_replace() {
    let vec = std::vec::Vec::from([32, 56]);
    let mut replace: HashMap<&str, i32> = HashMap::new();
    replace.insert("1", 96);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(vec)
        .add_struct(replace)
        .build()
        .unwrap();
    let value0 = config.get("0");
    assert_eq!(value0.unwrap().as_str(), "32");
    let value1 = config.get("1");
    assert_eq!(value1.unwrap().as_str(), "96");
}

#[test]
fn load_vec_merge() {
    let vec = std::vec::Vec::from([32, 56]);
    let mut merge: HashMap<&str, i32> = HashMap::new();
    merge.insert("2", 96);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(vec)
        .add_struct(merge)
        .build()
        .unwrap();
    let value0 = config.get("0");
    assert_eq!(value0.unwrap().as_str(), "32");
    let value1 = config.get("1");
    assert_eq!(value1.unwrap().as_str(), "56");
    let value1 = config.get("2");
    assert_eq!(value1.unwrap().as_str(), "96");
}

#[test]
fn load_vec_merge_to_far() {
    let vec = std::vec::Vec::from([32, 56]);
    let mut merge: HashMap<&str, i32> = HashMap::new();
    merge.insert("3", 96);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(vec)
        .add_struct(merge)
        .build()
        .unwrap();
    let value0 = config.get("0");
    assert_eq!(value0.unwrap().as_str(), "32");
    let value1 = config.get("1");
    assert_eq!(value1.unwrap().as_str(), "56");
    let value1 = config.get("3");
    assert_eq!(value1.unwrap().as_str(), "96");
}

#[test]
fn load_vec_merge_too_far_and_reify() {
    #[derive(Deserialize, Serialize, Clone, Debug)]
    #[serde(rename_all(serialize = "PascalCase"))]
    #[serde(rename_all(deserialize = "PascalCase"))]
    struct ArrayI32 {
        values: std::vec::Vec<i32>
    }

    let opts = ArrayI32 {
        values: std::vec::Vec::from([1, 2]),
    };

    let mut merge: HashMap<&str, i32> = HashMap::new();
    merge.insert("Values:3", 96);

    let config = DefaultConfigurationBuilder::new()
        .add_struct(opts.clone())
        .add_struct(merge)
        .build()
        .unwrap();
    let value0 = config.get("Values:0");
    assert_eq!(value0.unwrap().as_str(), "1");
    let value1 = config.get("Values:1");
    assert_eq!(value1.unwrap().as_str(), "2");
    let value1 = config.get("Values:3");
    assert_eq!(value1.unwrap().as_str(), "96");

    let loaded: ArrayI32 = config.reify();
    assert_eq!(loaded.values[0], 1);
    assert_eq!(loaded.values[1], 2);
    assert_eq!(loaded.values[2], 96);
}

