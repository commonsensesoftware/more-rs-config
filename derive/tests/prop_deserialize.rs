use proptest::prelude::*;

// All fields have #[serde(default)] so any subset can be omitted.
#[derive(config_derive::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
struct AllDefaults {
    #[serde(default)]
    name: String,
    #[serde(default)]
    count: u32,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    ratio: f64,
}

// Multiple field types for round-trip testing.
#[derive(config_derive::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
struct MultiField {
    label: String,
    value: i64,
    flag: bool,
    score: f64,
}

/// Strategy that generates f64 values which survive JSON round-trip exactly. Uses integer-scaled values that have exact
/// decimal representations.
fn json_safe_f64() -> impl Strategy<Value = f64> {
    // Generate f64 from integer numerator and small denominator to ensure exact JSON round-trip.
    // Values like n/1, n/2, n/4, n/5, n/8, n/10, etc. are exactly representable.
    prop_oneof![
        // Integers (always exact)
        (-1_000_000i64..1_000_000i64).prop_map(|n| n as f64),
        // Halves, quarters, eighths
        (-1_000_000i64..1_000_000i64).prop_map(|n| n as f64 / 2.0),
        (-1_000_000i64..1_000_000i64).prop_map(|n| n as f64 / 4.0),
        (-1_000_000i64..1_000_000i64).prop_map(|n| n as f64 / 8.0),
        // Powers of two scaled
        (-1_000_000i64..1_000_000i64).prop_map(|n| n as f64 / 16.0),
        // Small decimals (tenths, hundredths via integer division by powers of 2)
        (-1_000_000i64..1_000_000i64).prop_map(|n| n as f64 / 256.0),
    ]
}

// Generate random subsets of fields to include in JSON. Verify that omitted fields get their Default::default() values.
proptest! {
    #[test]
    fn deserialize_should_process_included_fields_and_have_defaults_for_absent_fields(
        include_name in any::<bool>(),
        include_count in any::<bool>(),
        include_enabled in any::<bool>(),
        include_ratio in any::<bool>(),
        name in "[a-zA-Z0-9]{0,20}",
        count in any::<u32>(),
        enabled in any::<bool>(),
        ratio in json_safe_f64(),
    ) {
        // arrange
        let mut fields = Vec::new();

        if include_name {
            fields.push(format!(r#""name":"{}""#, name));
        }

        if include_count {
            fields.push(format!(r#""count":{}"#, count));
        }

        if include_enabled {
            fields.push(format!(r#""enabled":{}"#, enabled));
        }

        if include_ratio {
            let ratio_json = serde_json::to_string(&ratio).unwrap();
            fields.push(format!(r#""ratio":{}"#, ratio_json));
        }

        // act
        let json = format!("{{{}}}", fields.join(","));
        let result: AllDefaults = serde_json::from_str(&json).unwrap();

        // assert
        if include_name {
            prop_assert_eq!(&result.name, &name);
        } else {
            prop_assert_eq!(&result.name, &String::default());
        }

        if include_count {
            prop_assert_eq!(result.count, count);
        } else {
            prop_assert_eq!(result.count, u32::default());
        }

        if include_enabled {
            prop_assert_eq!(result.enabled, enabled);
        } else {
            prop_assert_eq!(result.enabled, bool::default());
        }

        if include_ratio {
            prop_assert_eq!(result.ratio, ratio);
        } else {
            prop_assert_eq!(result.ratio, f64::default());
        }
    }
}

// Generate complete random values for all fields, serialize to JSON, deserialize back, and verify equality.
proptest! {
    #[test]
    fn deserialize_should_roundtrip_using_json(
        label in "[a-zA-Z0-9]{0,50}",
        value in any::<i64>(),
        flag in any::<bool>(),
        score in json_safe_f64(),
    ) {
        // arrange
        let original = MultiField {
            label: label.clone(),
            value,
            flag,
            score,
        };

        let json = serde_json::to_string(&original).unwrap();

        // act
        let deserialized: MultiField = serde_json::from_str(&json).unwrap();

        // assert
        prop_assert_eq!(&deserialized, &original);
    }
}
