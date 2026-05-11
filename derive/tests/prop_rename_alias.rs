use config_derive::Deserialize;
use proptest::prelude::*;

/// Struct with `rename_all = "camelCase"` to test that both `deserialize()` and `deserialize_in_place()` match the
/// transformed (camelCase) key names.
#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RenamedCamelCase {
    server_port: u16,
    host_name: String,
    max_retry_count: u32,
    is_enabled: bool,
}

/// Struct with alias attributes to test that both primary name and alias are accepted.
#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
struct WithAliases {
    #[serde(alias = "user_name")]
    name: String,
    #[serde(alias = "server_port")]
    port: u16,
    #[serde(alias = "is_active")]
    active: bool,
}

impl Default for WithAliases {
    fn default() -> Self {
        Self {
            name: String::new(),
            port: 0,
            active: false,
        }
    }
}

proptest! {
    #[test]
    fn deserialize_should_process_camel_case_fields(
        server_port in any::<u16>(),
        host_name in "[a-zA-Z][a-zA-Z0-9.\\-]{0,30}",
        max_retry_count in any::<u32>(),
        is_enabled in any::<bool>(),
    ) {
        // arrange
        let json = format!(
            r#"{{"serverPort": {}, "hostName": {:?}, "maxRetryCount": {}, "isEnabled": {}}}"#,
            server_port, host_name, max_retry_count, is_enabled
        );

        // act
        let result: RenamedCamelCase = serde_json::from_str(&json).unwrap();

        // assert
        prop_assert_eq!(result.server_port, server_port);
        prop_assert_eq!(&result.host_name, &host_name);
        prop_assert_eq!(result.max_retry_count, max_retry_count);
        prop_assert_eq!(result.is_enabled, is_enabled);
    }

    #[test]
    fn deserialize_in_place_should_process_camel_case_fields(
        server_port in any::<u16>(),
        host_name in "[a-zA-Z][a-zA-Z0-9.\\-]{0,30}",
        max_retry_count in any::<u32>(),
        is_enabled in any::<bool>(),
    ) {
        // arrange
        let json = format!(
            r#"{{"serverPort": {}, "hostName": {:?}, "maxRetryCount": {}, "isEnabled": {}}}"#,
            server_port, host_name, max_retry_count, is_enabled
        );
        let mut place = RenamedCamelCase {
            server_port: 0,
            host_name: String::new(),
            max_retry_count: 0,
            is_enabled: false,
        };
        let mut de = serde_json::Deserializer::from_str(&json);

        // act
        serde::Deserialize::deserialize_in_place(&mut de, &mut place).unwrap();

        // assert
        prop_assert_eq!(place.server_port, server_port);
        prop_assert_eq!(&place.host_name, &host_name);
        prop_assert_eq!(place.max_retry_count, max_retry_count);
        prop_assert_eq!(place.is_enabled, is_enabled);
    }

    #[test]
    fn deserialize_should_process_aliased_fields(
        name_val in "[a-zA-Z][a-zA-Z0-9 ]{0,20}",
        port_val in any::<u16>(),
        active_val in any::<bool>(),
        use_alias_for_name in any::<bool>(),
        use_alias_for_port in any::<bool>(),
        use_alias_for_active in any::<bool>(),
    ) {
        // arrange
        let name_key = if use_alias_for_name { "user_name" } else { "name" };
        let port_key = if use_alias_for_port { "server_port" } else { "port" };
        let active_key = if use_alias_for_active { "is_active" } else { "active" };
        let json = format!(
            r#"{{"{name_key}": {:?}, "{port_key}": {port_val}, "{active_key}": {active_val}}}"#,
            name_val,
        );

        // act
        let result: WithAliases = serde_json::from_str(&json).unwrap();

        // assert
        prop_assert_eq!(&result.name, &name_val);
        prop_assert_eq!(result.port, port_val);
        prop_assert_eq!(result.active, active_val);
    }

    #[test]
    fn deserialize_in_place_should_process_aliased_fields(
        name_val in "[a-zA-Z][a-zA-Z0-9 ]{0,20}",
        port_val in any::<u16>(),
        active_val in any::<bool>(),
        use_alias_for_name in any::<bool>(),
        use_alias_for_port in any::<bool>(),
        use_alias_for_active in any::<bool>(),
    ) {
        // arrange
        let name_key = if use_alias_for_name { "user_name" } else { "name" };
        let port_key = if use_alias_for_port { "server_port" } else { "port" };
        let active_key = if use_alias_for_active { "is_active" } else { "active" };
        let json = format!(
            r#"{{"{name_key}": {:?}, "{port_key}": {port_val}, "{active_key}": {active_val}}}"#,
            name_val,
        );
        let mut place = WithAliases::default();
        let mut de = serde_json::Deserializer::from_str(&json);

        // act
        serde::Deserialize::deserialize_in_place(&mut de, &mut place).unwrap();

        // assert
        prop_assert_eq!(&place.name, &name_val);
        prop_assert_eq!(place.port, port_val);
        prop_assert_eq!(place.active, active_val);
    }
}
