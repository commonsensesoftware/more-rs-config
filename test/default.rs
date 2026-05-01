use config::{mem, prelude::*};
use std::collections::HashMap;
use test_case::test_case;

#[test]
fn build_should_load_and_combine_different_configuration_sources() {
    // arrange
    let source1 = mem::Source::new(&[("Mem1:KeyInMem1", "ValueInMem1")]);
    let source2 = mem::Source::new(&[("Mem2:KeyInMem2", "ValueInMem2")]);
    let source3 = mem::Source::new(&[("Mem3:KeyInMem3", "ValueInMem3")]);
    let mut builder = config::builder();

    builder.add(source1);
    builder.add(source2);
    builder.add(source3);

    // act
    let config = builder.build().load().unwrap();

    // assert
    assert_eq!(config.get("mem1:keyinmem1"), Some("ValueInMem1"));
    assert_eq!(config.get("Mem2:KeyInMem2"), Some("ValueInMem2"));
    assert_eq!(config.get("MEM3:KEYINMEM3"), Some("ValueInMem3"));
}

#[test]
fn add_configuration_should_chain_configurations() {
    // arrange
    let source1 = mem::Source::new(&[("Mem1:KeyInMem1", "ValueInMem1")]);
    let source2 = mem::Source::new(&[("Mem2:KeyInMem2", "ValueInMem2")]);
    let source3 = mem::Source::new(&[("Mem3:KeyInMem3", "ValueInMem3")]);
    let mut builder = config::builder();

    builder.add(source1);
    builder.add(source2);
    builder.add(source3);

    let other = builder.build().load().unwrap();
    let builder = config::builder().add_configuration(other);

    // act
    let config = builder.build().load().unwrap();

    // assert
    assert_eq!(config.get("mem1:keyinmem1"), Some("ValueInMem1"));
    assert_eq!(config.get("Mem2:KeyInMem2"), Some("ValueInMem2"));
    assert_eq!(config.get("MEM3:KEYINMEM3"), Some("ValueInMem3"));
    assert_eq!(config.get("Nonexistent"), None);
}

#[test]
fn iter_should_flatten_into_hashmap() {
    // arrange
    let source1 = mem::Source::new(&[
        ("Mem1", "Value1"),
        ("Mem1:", "NoKeyValue1"),
        ("Mem1:KeyInMem1", "ValueInMem1"),
        ("Mem1:KeyInMem1:Deep1", "ValueDeep1"),
    ]);
    let source2 = mem::Source::new(&[
        ("Mem2", "Value2"),
        ("Mem2:", "NoKeyValue2"),
        ("Mem2:KeyInMem2", "ValueInMem2"),
        ("Mem2:KeyInMem2:Deep2", "ValueDeep2"),
    ]);
    let source3 = mem::Source::new(&[
        ("Mem3", "Value3"),
        ("Mem3:", "NoKeyValue3"),
        ("Mem3:KeyInMem3", "ValueInMem3"),
        ("Mem3:KeyInMem3:Deep3", "ValueDeep3"),
    ]);
    let mut builder = config::builder();

    builder.add(source1);
    builder.add(source2);
    builder.add(source3);

    let config = builder.build().load().unwrap();

    // act
    let map = config.into_iter().collect::<HashMap<_, _>>();

    // assert
    assert_eq!(map["Mem1"], "Value1");
    assert_eq!(map["Mem1:"], "NoKeyValue1");
    assert_eq!(map["Mem1:KeyInMem1"], "ValueInMem1");
    assert_eq!(map["Mem1:KeyInMem1:Deep1"], "ValueDeep1");
    assert_eq!(map["Mem2"], "Value2");
    assert_eq!(map["Mem2:"], "NoKeyValue2");
    assert_eq!(map["Mem2:KeyInMem2"], "ValueInMem2");
    assert_eq!(map["Mem2:KeyInMem2:Deep2"], "ValueDeep2");
    assert_eq!(map["Mem3"], "Value3");
    assert_eq!(map["Mem3:"], "NoKeyValue3");
    assert_eq!(map["Mem3:KeyInMem3"], "ValueInMem3");
    assert_eq!(map["Mem3:KeyInMem3:Deep3"], "ValueDeep3");
}

#[test]
fn chained_iter_should_flatten_into_hashmap() {
    // arrange
    let source1 = mem::Source::new(&[
        ("Mem1", "Value1"),
        ("Mem1:", "NoKeyValue1"),
        ("Mem1:KeyInMem1", "ValueInMem1"),
        ("Mem1:KeyInMem1:Deep1", "ValueDeep1"),
    ]);
    let source2 = mem::Source::new(&[
        ("Mem2", "Value2"),
        ("Mem2:", "NoKeyValue2"),
        ("Mem2:KeyInMem2", "ValueInMem2"),
        ("Mem2:KeyInMem2:Deep2", "ValueDeep2"),
    ]);
    let source3 = mem::Source::new(&[
        ("Mem3", "Value3"),
        ("Mem3:", "NoKeyValue3"),
        ("Mem3:KeyInMem3", "ValueInMem3"),
        ("Mem3:KeyInMem3:Deep3", "ValueDeep3"),
    ]);
    let mut builder = config::builder();

    builder.add(source1);
    builder.add(source2);

    let other = builder.build().load().unwrap();
    let mut builder = config::builder().add_configuration(other);

    builder.add(source3);

    let config = builder.build().load().unwrap();

    // act
    let map = config.into_iter().collect::<HashMap<_, _>>();

    // assert
    assert_eq!(map["Mem1"], "Value1");
    assert_eq!(map["Mem1:"], "NoKeyValue1");
    assert_eq!(map["Mem1:KeyInMem1"], "ValueInMem1");
    assert_eq!(map["Mem1:KeyInMem1:Deep1"], "ValueDeep1");
    assert_eq!(map["Mem2"], "Value2");
    assert_eq!(map["Mem2:"], "NoKeyValue2");
    assert_eq!(map["Mem2:KeyInMem2"], "ValueInMem2");
    assert_eq!(map["Mem2:KeyInMem2:Deep2"], "ValueDeep2");
    assert_eq!(map["Mem3"], "Value3");
    assert_eq!(map["Mem3:"], "NoKeyValue3");
    assert_eq!(map["Mem3:KeyInMem3"], "ValueInMem3");
    assert_eq!(map["Mem3:KeyInMem3:Deep3"], "ValueDeep3");
}

// #[test]
// fn iter_should_strip_key_from_children() {
//     // arrange
//     let source1 = mem::Source::new(&[
//         ("Mem1", "Value1"),
//         ("Mem1:", "NoKeyValue1"),
//         ("Mem1:KeyInMem1", "ValueInMem1"),
//         ("Mem1:KeyInMem1:Deep1", "ValueDeep1"),
//     ]);
//     let source2 = mem::Source::new(&[
//         ("Mem2", "Value2"),
//         ("Mem2:", "NoKeyValue2"),
//         ("Mem2:KeyInMem2", "ValueInMem2"),
//         ("Mem2:KeyInMem2:Deep2", "ValueDeep2"),
//     ]);
//     let source3 = mem::Source::new(&[
//         ("Mem3", "Value3"),
//         ("Mem3:", "NoKeyValue3"),
//         ("Mem3:KeyInMem3", "ValueInMem3"),
//         ("Mem3:KeyInMem4", "ValueInMem4"),
//         ("Mem3:KeyInMem3:Deep3", "ValueDeep3"),
//         ("Mem3:KeyInMem3:Deep4", "ValueDeep4"),
//     ]);
//     let mut builder = config::builder();

//     builder.add(source1);
//     builder.add(source2);
//     builder.add(source3);

//     let config = builder.build().load().unwrap();

//     // act
//     let map1: HashMap<_, _> = config.section("Mem1").iter(Some(Relative)).collect();
//     let map2: HashMap<_, _> = config.section("Mem2").iter(Some(Relative)).collect();
//     let map3: HashMap<_, _> = config.section("Mem3").iter(Some(Relative)).collect();

//     // assert
//     assert_eq!(map1.len(), 3);
//     assert_eq!(map1[""].as_str(), "NoKeyValue1");
//     assert_eq!(map1["KeyInMem1"].as_str(), "ValueInMem1");
//     assert_eq!(map1["KeyInMem1:Deep1"].as_str(), "ValueDeep1");
//     assert_eq!(map2.len(), 3);
//     assert_eq!(map2[""].as_str(), "NoKeyValue2");
//     assert_eq!(map2["KeyInMem2"].as_str(), "ValueInMem2");
//     assert_eq!(map2["KeyInMem2:Deep2"].as_str(), "ValueDeep2");
//     assert_eq!(map3.len(), 5);
//     assert_eq!(map3[""].as_str(), "NoKeyValue3");
//     assert_eq!(map3["KeyInMem3"].as_str(), "ValueInMem3");
//     assert_eq!(map3["KeyInMem4"].as_str(), "ValueInMem4");
//     assert_eq!(map3["KeyInMem3:Deep3"].as_str(), "ValueDeep3");
//     assert_eq!(map3["KeyInMem3:Deep4"].as_str(), "ValueDeep4");
// }

#[test]
fn new_configuration_provider_should_override_old_one_when_key_is_duplicated() {
    // arrange
    let source1 = mem::Source::new(&[("Key1:Key2", "ValueInMem1")]);
    let source2 = mem::Source::new(&[("Key1:Key2", "ValueInMem2")]);
    let mut builder = config::builder();

    builder.add(source1);
    builder.add(source2);

    // act
    let config = builder.build().load().unwrap();

    // assert
    assert_eq!(config.get("Key1:Key2"), Some("ValueInMem2"));
}

#[test]
fn new_configuration_root_should_be_built_from_existing_with_duplicate_keys() {
    // arrange
    let other = config::builder()
        .add_in_memory(&[("keya:keyb", "valueA")])
        .add_in_memory(&[("KEYA:KEYB", "valueB")])
        .build()
        .load()
        .unwrap();

    // act
    let config = config::builder()
        .add_in_memory(&other.into_iter().collect::<Vec<_>>())
        .build()
        .load()
        .unwrap();

    // assert
    assert_eq!(config.get("keya:keyb"), Some("valueB"));
}

#[test]
fn section_should_return_parts_from_root_configuration() {
    // arrange
    let source1 = mem::Source::new(&[("Data:DB1:Connection1", "MemVal1"), ("Data:DB1:Connection2", "MemVal2")]);
    let source2 = mem::Source::new(&[("DataSource:DB2:Connection", "MemVal3")]);
    let source3 = mem::Source::new(&[("Data", "MemVal4")]);
    let mut builder = config::builder();

    builder.add(source1);
    builder.add(source2);
    builder.add(source3);

    let config = builder.build().load().unwrap();

    // act
    let section = config.section("Data");

    // assert
    assert_eq!(section.get("DB1:Connection1"), Some("MemVal1"));
    assert_eq!(section.get("DB1:Connection2"), Some("MemVal2"));
    assert_eq!(section.value(), "MemVal4");
    assert_eq!(section.get("DB2:Connection"), None);
    assert_eq!(section.get("Source:DB2:Connection"), None);
}

#[ignore]
#[test]
fn section_should_return_children() {
    // arrange
    let source1 = mem::Source::new(&[("Data:DB1:Connection1", "MemVal1"), ("Data:DB1:Connection2", "MemVal2")]);
    let source2 = mem::Source::new(&[("Data:DB2Connection", "MemVal3")]);
    let source3 = mem::Source::new(&[("DataSource:DB3:Connection", "MemVal4")]);
    let mut builder = config::builder();

    builder.add(source1);
    builder.add(source2);
    builder.add(source3);

    let config = builder.build().load().unwrap();

    // act
    let sections = config.section("Data").sections();

    // assert
    assert_eq!(sections.len(), 2);
    assert_eq!(
        sections.iter().find(|s| s.key() == "DB1").unwrap().get("Connection1"),
        Some("MemVal1")
    );
    assert_eq!(
        sections.iter().find(|s| s.key() == "DB1").unwrap().get("Connection2"),
        Some("MemVal2")
    );
    assert_eq!(
        sections.iter().find(|s| s.key() == "DB2Connection").unwrap().value(),
        "MemVal3"
    );
    assert!(sections.iter().find(|s| s.key() == "DB3").is_none());
}

#[test_case("Value1", true ; "should exist with value")]
#[test_case("", false ; "should not exist with empty value")]
fn section_without_children(value: &str, expected: bool) {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Mem1", value)])
        .build()
        .load()
        .unwrap();

    // act
    let section = config.section("Mem1");

    // assert
    assert_eq!(section.exists(), expected);
}

#[test]
fn section_with_children_should_exist() {
    // arrange
    let root = config::builder()
        .add_in_memory(&[
            ("Mem1:KeyInMem1", "ValueInMem1"),
            ("Mem1:KeyInMem1:Deep1", "ValueDeep1"),
            ("Mem2:KeyInMem2:Deep1", "ValueDeep2"),
        ])
        .build();

    // act
    let config = root.load().unwrap();

    // assert
    assert!(config.section("Mem1").exists());
    assert!(config.section("Mem2").exists());
    assert!(!config.section("Mem3").exists());
}

#[ignore]
#[test]
fn key_starting_with_colon_means_first_section_has_empty_name() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[(":Key2", "value")])
        .build()
        .load()
        .unwrap();

    // act
    let sections = config.sections();

    // assert
    assert_eq!(sections.len(), 1);
    assert_eq!("", sections[0].key());
    assert_eq!(sections[0].sections().len(), 1);
    assert_eq!(sections[0].sections()[0].key(), "Key2");
}

#[ignore]
#[test]
fn key_ending_with_colon_means_last_section_has_empty_name() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Key1:", "value")])
        .build()
        .load()
        .unwrap();

    // act
    let sections = config.sections();

    // assert
    assert_eq!(sections.len(), 1);
    assert_eq!("Key1", sections[0].key());
    assert_eq!(sections[0].sections().len(), 1);
    assert_eq!(sections[0].sections()[0].key(), "");
}

#[ignore]
#[test]
fn key_ending_with_double_colon_has_section_with_empty_name() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Key1::Key3", "value")])
        .build()
        .load()
        .unwrap();

    // act
    let sections = config.sections();

    // assert
    assert_eq!(sections.len(), 1);
    assert_eq!("Key1", sections[0].key());
    assert_eq!(sections[0].sections().len(), 1);
    assert_eq!(sections[0].sections()[0].key(), "");
    assert_eq!(sections[0].sections()[0].sections().len(), 1);
    assert_eq!(sections[0].sections()[0].sections()[0].key(), "Key3");
}
