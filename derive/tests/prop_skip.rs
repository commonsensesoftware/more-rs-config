use proptest::prelude::*;

/// Test struct with some fields marked as skipped. `skipped_num` uses #[serde(skip)] and `skipped_str` uses
/// #[serde(skip_deserializing)]. Both should never be assigned from the deserializer.
#[derive(config_derive::Deserialize, Clone, Debug, PartialEq)]
struct SkipTest {
    name: String,
    value: u32,
    #[serde(skip)]
    skipped_num: u32,
    #[serde(skip_deserializing)]
    skipped_str: String,
}

fn deserialize_in_place_from_str(place: &mut SkipTest, json: &str) {
    let mut de = serde_json::Deserializer::from_str(json);
    serde::Deserialize::deserialize_in_place(&mut de, place).unwrap();
}

proptest! {
    #[test]
    fn deserialize_should_exclude_skipped_fields(
        name in "[a-zA-Z0-9]{0,20}",
        value in any::<u32>(),
        skipped_num_input in any::<u32>(),
        skipped_str_input in "[a-zA-Z0-9]{0,20}",
    ) {
        // arrange
        let json = format!(
            r#"{{"name":"{}","value":{},"skipped_num":{},"skipped_str":"{}"}}"#,
            name, value, skipped_num_input, skipped_str_input
        );

        // act
        let result: SkipTest = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(&result.name, &name);
        prop_assert_eq!(result.value, value);
        prop_assert_eq!(result.skipped_num, u32::default(),
            "skipped_num should be default (0), but was {} (JSON had {})",
            result.skipped_num, skipped_num_input);
        prop_assert_eq!(&result.skipped_str, &String::default(),
            "skipped_str should be default (\"\"), but was {:?} (JSON had {:?})",
            result.skipped_str, skipped_str_input);
    }
}

proptest! {
    #[test]
    fn deserialize_in_place_should_exclude_skipped_fields(
        name in "[a-zA-Z0-9]{0,20}",
        value in any::<u32>(),
        initial_skipped_num in 1u32..=u32::MAX,
        initial_skipped_str in "[a-zA-Z]{1,20}",
        json_skipped_num in any::<u32>(),
        json_skipped_str in "[a-zA-Z0-9]{0,20}",
    ) {
        // arrange
        let mut instance = SkipTest {
            name: String::from("original"),
            value: 0,
            skipped_num: initial_skipped_num,
            skipped_str: initial_skipped_str.clone(),
        };
        let json = format!(
            r#"{{"name":"{}","value":{},"skipped_num":{},"skipped_str":"{}"}}"#,
            name, value, json_skipped_num, json_skipped_str
        );

        // act
        deserialize_in_place_from_str(&mut instance, &json);

        // assert
        prop_assert_eq!(&instance.name, &name);
        prop_assert_eq!(instance.value, value);
        prop_assert_eq!(instance.skipped_num, initial_skipped_num,
            "skipped_num should remain {} but was {} (JSON tried to set {})",
            initial_skipped_num, instance.skipped_num, json_skipped_num);
        prop_assert_eq!(&instance.skipped_str, &initial_skipped_str,
            "skipped_str should remain {:?} but was {:?} (JSON tried to set {:?})",
            initial_skipped_str, instance.skipped_str, json_skipped_str);
    }
}
