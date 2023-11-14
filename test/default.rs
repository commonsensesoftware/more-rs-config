use config::{ext::*, *};
use std::collections::HashMap;
use test_case::test_case;

#[test]
fn build_should_load_and_combine_different_configuration_sources() {
    // arrange
    let source1 = MemoryConfigurationSource::new(
        [("Mem1:KeyInMem1", "ValueInMem1")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let source2 = MemoryConfigurationSource::new(
        [("Mem2:KeyInMem2", "ValueInMem2")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let source3 = MemoryConfigurationSource::new(
        [("Mem3:KeyInMem3", "ValueInMem3")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source1));
    builder.add(Box::new(source2));
    builder.add(Box::new(source3));

    // act
    let config = builder.build();

    // assert
    assert_eq!(*config.get("mem1:keyinmem1").unwrap(), "ValueInMem1");
    assert_eq!(*config.get("Mem2:KeyInMem2").unwrap(), "ValueInMem2");
    assert_eq!(*config.get("MEM3:KEYINMEM3").unwrap(), "ValueInMem3");
}

#[test]
fn add_configuration_should_chain_configurations() {
    // arrange
    let source1 = MemoryConfigurationSource::new(
        [("Mem1:KeyInMem1", "ValueInMem1")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let source2 = MemoryConfigurationSource::new(
        [("Mem2:KeyInMem2", "ValueInMem2")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let source3 = MemoryConfigurationSource::new(
        [("Mem3:KeyInMem3", "ValueInMem3")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source1));
    builder.add(Box::new(source2));
    builder.add(Box::new(source3));

    let root = builder.build();
    let mut builder2 = DefaultConfigurationBuilder::new();

    builder2.add_configuration(root.as_config());

    // act
    let config = builder2.build();

    // assert
    assert_eq!(*config.get("mem1:keyinmem1").unwrap(), "ValueInMem1");
    assert_eq!(*config.get("Mem2:KeyInMem2").unwrap(), "ValueInMem2");
    assert_eq!(*config.get("MEM3:KEYINMEM3").unwrap(), "ValueInMem3");
    assert!(config.get("Nonexistent").is_none());
}

#[test_case(false ; "with original path")]
#[test_case(true ; "with relative path")]
fn iter_should_flatten_into_hashmap(make_paths_relative: bool) {
    // arrange
    let source1 = MemoryConfigurationSource::new(
        [
            ("Mem1", "Value1"),
            ("Mem1:", "NoKeyValue1"),
            ("Mem1:KeyInMem1", "ValueInMem1"),
            ("Mem1:KeyInMem1:Deep1", "ValueDeep1"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let source2 = MemoryConfigurationSource::new(
        [
            ("Mem2", "Value2"),
            ("Mem2:", "NoKeyValue2"),
            ("Mem2:KeyInMem2", "ValueInMem2"),
            ("Mem2:KeyInMem2:Deep2", "ValueDeep2"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let source3 = MemoryConfigurationSource::new(
        [
            ("Mem3", "Value3"),
            ("Mem3:", "NoKeyValue3"),
            ("Mem3:KeyInMem3", "ValueInMem3"),
            ("Mem3:KeyInMem3:Deep3", "ValueDeep3"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source1));
    builder.add(Box::new(source2));
    builder.add(Box::new(source3));
    let root = builder.build();

    // act
    let map: HashMap<_, _> = root
        .iter_relative(make_paths_relative)
        .collect();

    // assert
    assert_eq!(&map["Mem1"], "Value1");
    assert_eq!(&map["Mem1:"], "NoKeyValue1");
    assert_eq!(&map["Mem1:KeyInMem1"], "ValueInMem1");
    assert_eq!(&map["Mem1:KeyInMem1:Deep1"], "ValueDeep1");
    assert_eq!(&map["Mem2"], "Value2");
    assert_eq!(&map["Mem2:"], "NoKeyValue2");
    assert_eq!(&map["Mem2:KeyInMem2"], "ValueInMem2");
    assert_eq!(&map["Mem2:KeyInMem2:Deep2"], "ValueDeep2");
    assert_eq!(&map["Mem3"], "Value3");
    assert_eq!(&map["Mem3:"], "NoKeyValue3");
    assert_eq!(&map["Mem3:KeyInMem3"], "ValueInMem3");
    assert_eq!(&map["Mem3:KeyInMem3:Deep3"], "ValueDeep3");
}

#[test_case(false ; "with original path")]
#[test_case(true ; "with relative path")]
fn chained_iter_should_flatten_into_hashmap(make_paths_relative: bool) {
    // arrange
    let source1 = MemoryConfigurationSource::new(
        [
            ("Mem1", "Value1"),
            ("Mem1:", "NoKeyValue1"),
            ("Mem1:KeyInMem1", "ValueInMem1"),
            ("Mem1:KeyInMem1:Deep1", "ValueDeep1"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let source2 = MemoryConfigurationSource::new(
        [
            ("Mem2", "Value2"),
            ("Mem2:", "NoKeyValue2"),
            ("Mem2:KeyInMem2", "ValueInMem2"),
            ("Mem2:KeyInMem2:Deep2", "ValueDeep2"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let source3 = MemoryConfigurationSource::new(
        [
            ("Mem3", "Value3"),
            ("Mem3:", "NoKeyValue3"),
            ("Mem3:KeyInMem3", "ValueInMem3"),
            ("Mem3:KeyInMem3:Deep3", "ValueDeep3"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source1));
    builder.add(Box::new(source2));

    let other = builder.build();
    let mut builder2 = DefaultConfigurationBuilder::new();

    builder2
        .add_configuration(other.as_config())
        .add(Box::new(source3));
    let root = builder2.build();

    // act
    let map: HashMap<_, _> = root
        .iter_relative(make_paths_relative)
        .collect();

    // assert
    assert_eq!(&map["Mem1"], "Value1");
    assert_eq!(&map["Mem1:"], "NoKeyValue1");
    assert_eq!(&map["Mem1:KeyInMem1"], "ValueInMem1");
    assert_eq!(&map["Mem1:KeyInMem1:Deep1"], "ValueDeep1");
    assert_eq!(&map["Mem2"], "Value2");
    assert_eq!(&map["Mem2:"], "NoKeyValue2");
    assert_eq!(&map["Mem2:KeyInMem2"], "ValueInMem2");
    assert_eq!(&map["Mem2:KeyInMem2:Deep2"], "ValueDeep2");
    assert_eq!(&map["Mem3"], "Value3");
    assert_eq!(&map["Mem3:"], "NoKeyValue3");
    assert_eq!(&map["Mem3:KeyInMem3"], "ValueInMem3");
    assert_eq!(&map["Mem3:KeyInMem3:Deep3"], "ValueDeep3");
}

#[test]
fn iter_should_strip_key_from_children() {
    // arrange
    let source1 = MemoryConfigurationSource::new(
        [
            ("Mem1", "Value1"),
            ("Mem1:", "NoKeyValue1"),
            ("Mem1:KeyInMem1", "ValueInMem1"),
            ("Mem1:KeyInMem1:Deep1", "ValueDeep1"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let source2 = MemoryConfigurationSource::new(
        [
            ("Mem2", "Value2"),
            ("Mem2:", "NoKeyValue2"),
            ("Mem2:KeyInMem2", "ValueInMem2"),
            ("Mem2:KeyInMem2:Deep2", "ValueDeep2"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let source3 = MemoryConfigurationSource::new(
        [
            ("Mem3", "Value3"),
            ("Mem3:", "NoKeyValue3"),
            ("Mem3:KeyInMem3", "ValueInMem3"),
            ("Mem3:KeyInMem4", "ValueInMem4"),
            ("Mem3:KeyInMem3:Deep3", "ValueDeep3"),
            ("Mem3:KeyInMem3:Deep4", "ValueDeep4"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source1));
    builder.add(Box::new(source2));
    builder.add(Box::new(source3));

    let config = builder.build();

    // act
    let map1: HashMap<_, _> = config
        .section("Mem1")
        .iter_relative(true)
        .collect();
    let map2: HashMap<_, _> = config
        .section("Mem2")
        .iter_relative(true)
        .collect();
    let map3: HashMap<_, _> = config
        .section("Mem3")
        .iter_relative(true)
        .collect();

    // assert
    assert_eq!(map1.len(), 3);
    assert_eq!(&map1[""], "NoKeyValue1");
    assert_eq!(&map1["KeyInMem1"], "ValueInMem1");
    assert_eq!(&map1["KeyInMem1:Deep1"], "ValueDeep1");
    assert_eq!(map2.len(), 3);
    assert_eq!(&map2[""], "NoKeyValue2");
    assert_eq!(&map2["KeyInMem2"], "ValueInMem2");
    assert_eq!(&map2["KeyInMem2:Deep2"], "ValueDeep2");
    assert_eq!(map3.len(), 5);
    assert_eq!(&map3[""], "NoKeyValue3");
    assert_eq!(&map3["KeyInMem3"], "ValueInMem3");
    assert_eq!(&map3["KeyInMem4"], "ValueInMem4");
    assert_eq!(&map3["KeyInMem3:Deep3"], "ValueDeep3");
    assert_eq!(&map3["KeyInMem3:Deep4"], "ValueDeep4");
}

#[test]
fn new_configuration_provider_should_override_old_one_when_key_is_duplicated() {
    // arrange
    let source1 = MemoryConfigurationSource::new(
        [("Key1:Key2", "ValueInMem1")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let source2 = MemoryConfigurationSource::new(
        [("Key1:Key2", "ValueInMem2")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source1));
    builder.add(Box::new(source2));

    // act
    let config = builder.build();

    // assert
    assert_eq!(*config.get("Key1:Key2").unwrap(), "ValueInMem2");
}

#[test]
fn new_configuration_root_should_be_built_from_existing_with_duplicate_keys() {
    // arrange
    let root1 = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [("keya:keyb", "valueA")]
                .iter()
                .map(|t| (t.0.to_owned(), t.1.to_owned()))
                .collect(),
        )
        .add_in_memory(
            [("KEYA:KEYB", "valueB")]
                .iter()
                .map(|t| (t.0.to_owned(), t.1.to_owned()))
                .collect(),
        )
        .build();

    // act
    let root2 = DefaultConfigurationBuilder::new()
        .add_in_memory(root1.iter().collect())
        .build();

    // assert
    assert_eq!(*root2.get("keya:keyb").unwrap(), "valueB");
}

#[test]
fn section_should_return_parts_from_root_configuration() {
    // arrange
    let source1 = MemoryConfigurationSource::new(
        [
            ("Data:DB1:Connection1", "MemVal1"),
            ("Data:DB1:Connection2", "MemVal2"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let source2 = MemoryConfigurationSource::new(
        [("DataSource:DB2:Connection", "MemVal3")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let source3 = MemoryConfigurationSource::new(
        [("Data", "MemVal4")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source1));
    builder.add(Box::new(source2));
    builder.add(Box::new(source3));

    let config = builder.build();

    // act
    let section = config.section("Data");

    // assert
    assert_eq!(
        *section.get("DB1:Connection1").unwrap(),
        "MemVal1"
    );
    assert_eq!(
        *section.get("DB1:Connection2").unwrap(),
        "MemVal2"
    );
    assert_eq!(*section.value(), "MemVal4");
    assert!(section.get("DB2:Connection").is_none());
    assert!(section.get("Source:DB2:Connection").is_none());
}

#[test]
fn section_should_return_children() {
    // arrange
    let source1 = MemoryConfigurationSource::new(
        [
            ("Data:DB1:Connection1", "MemVal1"),
            ("Data:DB1:Connection2", "MemVal2"),
        ]
        .iter()
        .map(|t| (t.0.to_owned(), t.1.to_owned()))
        .collect(),
    );
    let source2 = MemoryConfigurationSource::new(
        [("Data:DB2Connection", "MemVal3")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let source3 = MemoryConfigurationSource::new(
        [("DataSource:DB3:Connection", "MemVal4")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source1));
    builder.add(Box::new(source2));
    builder.add(Box::new(source3));

    let config = builder.build();

    // act
    let sections = config.section("Data").children();

    // assert
    assert_eq!(sections.len(), 2);
    assert_eq!(
        *sections
            .iter()
            .find(|s| s.key() == "DB1")
            .unwrap()
            .get("Connection1")
            .unwrap(),
        "MemVal1"
    );
    assert_eq!(
        *sections
            .iter()
            .find(|s| s.key() == "DB1")
            .unwrap()
            .get("Connection2")
            .unwrap(),
        "MemVal2"
    );
    assert_eq!(
        *sections
            .iter()
            .find(|s| s.key() == "DB2Connection")
            .unwrap()
            .value(),
        "MemVal3"
    );
    assert!(sections.iter().find(|s| s.key() == "DB3").is_none());
}

#[test_case("Value1", true ; "should exist with value")]
#[test_case("", false ; "should not exist with empty value")]
fn section_without_children(value: &str, expected: bool) {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [("Mem1", value)]
                .iter()
                .map(|t| (t.0.to_owned(), t.1.to_owned()))
                .collect(),
        )
        .build();

    // act
    let section = config.section("Mem1");

    // assert
    assert_eq!(section.exists(), expected);
}

#[test]
fn section_with_children_should_exist() {
    // arrange

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
                ("Mem1:KeyInMem1", "ValueInMem1"),
                ("Mem1:KeyInMem1:Deep1", "ValueDeep1"),
                ("Mem2:KeyInMem2:Deep1", "ValueDeep2"),
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build();

    // assert
    assert!(config.section("Mem1").exists());
    assert!(config.section("Mem2").exists());
    assert!(!config.section("Mem3").exists());
}

#[test]
fn key_starting_with_colon_means_first_section_has_empty_name() {
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [(":Key2", "value")]
                .iter()
                .map(|t| (t.0.to_owned(), t.1.to_owned()))
                .collect(),
        )
        .build();

    // act
    let children = config.children();

    // assert
    assert_eq!(children.len(), 1);
    assert_eq!("", children[0].key());
    assert_eq!(children[0].children().len(), 1);
    assert_eq!(children[0].children()[0].key(), "Key2");
}

#[test]
fn key_ending_with_colon_means_last_section_has_empty_name() {
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [("Key1:", "value")]
                .iter()
                .map(|t| (t.0.to_owned(), t.1.to_owned()))
                .collect(),
        )
        .build();

    // act
    let children = config.children();

    // assert
    assert_eq!(children.len(), 1);
    assert_eq!("Key1", children[0].key());
    assert_eq!(children[0].children().len(), 1);
    assert_eq!(children[0].children()[0].key(), "");
}

#[test]
fn key_ending_with_double_colon_has_section_with_empty_name() {
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [("Key1::Key3", "value")]
                .iter()
                .map(|t| (t.0.to_owned(), t.1.to_owned()))
                .collect(),
        )
        .build();

    // act
    let children = config.children();

    // assert
    assert_eq!(children.len(), 1);
    assert_eq!("Key1", children[0].key());
    assert_eq!(children[0].children().len(), 1);
    assert_eq!(children[0].children()[0].key(), "");
    assert_eq!(children[0].children()[0].children().len(), 1);
    assert_eq!(children[0].children()[0].children()[0].key(), "Key3");
}
