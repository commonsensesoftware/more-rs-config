use config::{ext::*, *};
use std::env::{set_var, temp_dir};
use std::fs::{remove_file, File};
use std::io::Write;

#[test]
fn should_load_chained_settings_from_yaml_file_and_env() {
    // arrange
    let yaml = r#"
service:
  enabled: false
feature:
  nativeCopy:
    disabled: true
"#;
    let path = temp_dir().join("test_settings_1.yaml");
    let mut file = File::create(&path).unwrap();

    file.write_all(yaml.as_bytes()).unwrap();

    let expected = "true";

    set_var("Feature__NativeCopy__Disabled", expected);

    let config = DefaultConfigurationBuilder::new()
        .add_yaml_file(&path)
        .add_env_vars()
        .build()
        .unwrap();
    let section = config.section("Feature").section("NativeCopy");

    // act
    let value = section.get("Disabled");

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }

    assert_eq!(value.unwrap().as_str(), "true");
}
