use proptest::prelude::*;

/// Test struct with several fields of different types for property testing.
#[derive(Clone, Debug, PartialEq, config_derive::Deserialize)]
struct TestConfig {
    name: String,
    count: u32,
    enabled: bool,
    ratio: f64,
}

/// Represents which fields are included in a partial update.
#[derive(Clone, Debug)]
struct FieldSubset {
    include_name: bool,
    include_count: bool,
    include_enabled: bool,
    include_ratio: bool,
}

/// Build a partial JSON string containing only the fields indicated by the subset, using the "new" values. Uses
/// serde_json for proper value serialization. Also returns the expected f64 value for ratio after JSON roundtrip.
fn build_partial_json(new: &TestConfig, subset: &FieldSubset) -> (String, Option<f64>) {
    use serde_json::{Map, Value};

    let mut map = Map::new();

    if subset.include_name {
        map.insert("name".to_string(), Value::String(new.name.clone()));
    }
    if subset.include_count {
        map.insert("count".to_string(), Value::from(new.count));
    }
    if subset.include_enabled {
        map.insert("enabled".to_string(), Value::Bool(new.enabled));
    }
    if subset.include_ratio {
        map.insert("ratio".to_string(), serde_json::to_value(new.ratio).unwrap());
    }

    let json_str = serde_json::to_string(&Value::Object(map)).unwrap();

    let ratio_expected = if subset.include_ratio {
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        Some(parsed["ratio"].as_f64().unwrap())
    } else {
        None
    };

    (json_str, ratio_expected)
}

/// Strategy to generate arbitrary TestConfig instances.
fn arb_test_config() -> impl Strategy<Value = TestConfig> {
    (
        "[ -~]{0,50}", // printable ASCII strings up to 50 chars
        any::<u32>(),
        any::<bool>(),
        // Use finite f64 values only — JSON cannot represent NaN/Infinity.
        // Filter the standard f64 strategy to only finite values.
        any::<f64>().prop_filter("must be finite", |v| v.is_finite()),
    )
        .prop_map(|(name, count, enabled, ratio)| TestConfig {
            name,
            count,
            enabled,
            ratio,
        })
}

/// Strategy to generate a random field subset with at least one field included.
fn arb_field_subset() -> impl Strategy<Value = FieldSubset> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>())
        .prop_filter("at least one field must be included", |(a, b, c, d)| {
            *a || *b || *c || *d
        })
        .prop_map(|(a, b, c, d)| FieldSubset {
            include_name: a,
            include_count: b,
            include_enabled: c,
            include_ratio: d,
        })
}

proptest! {
    #[test]
    fn deserialize_in_place_should_update_fields_present_in_json(
        initial in arb_test_config(),
        new_values in arb_test_config(),
        subset in arb_field_subset(),
    ) {
        // arrange
        let (json, ratio_expected) = build_partial_json(&new_values, &subset);
        let mut instance = initial.clone();
        let mut deserializer = serde_json::Deserializer::from_str(&json);

        // act
        serde::Deserialize::deserialize_in_place(&mut deserializer, &mut instance).unwrap();

        // assert
        if subset.include_name {
            prop_assert_eq!(&instance.name, &new_values.name,
                "name should be updated when present in JSON");
        } else {
            prop_assert_eq!(&instance.name, &initial.name,
                "name should be preserved when absent from JSON");
        }

        if subset.include_count {
            prop_assert_eq!(instance.count, new_values.count,
                "count should be updated when present in JSON");
        } else {
            prop_assert_eq!(instance.count, initial.count,
                "count should be preserved when absent from JSON");
        }

        if subset.include_enabled {
            prop_assert_eq!(instance.enabled, new_values.enabled,
                "enabled should be updated when present in JSON");
        } else {
            prop_assert_eq!(instance.enabled, initial.enabled,
                "enabled should be preserved when absent from JSON");
        }

        if let Some(expected) = ratio_expected {
            prop_assert_eq!(instance.ratio.to_bits(), expected.to_bits(),
                "ratio should be updated when present in JSON: got {:?} expected {:?}",
                instance.ratio, expected);
        } else {
            prop_assert_eq!(instance.ratio.to_bits(), initial.ratio.to_bits(),
                "ratio should be preserved when absent from JSON: got {:?} expected {:?}",
                instance.ratio, initial.ratio);
        }
    }
}
