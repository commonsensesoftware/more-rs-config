use config::{ext::*, *};
use serde::Deserialize;
use std::env::var;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Default, Deserialize)]
struct ContactOptions {
    name: String,
    primary: bool,
    phones: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct FileCopySettings {
    #[serde(default, alias = "UseSfpCopy")]
    use_sfp_copy: bool,
}

#[test]
fn get_as_should_deserialize_configuration_to_options() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
                ("name", "John Doe"),
                ("primary", "true"),
                ("phones:0", "+44 1234567"),
                ("phones:1", "+44 2345678"),
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build()
        .to_config();

    // act
    let options: ContactOptions = config.get_as();

    // assert
    assert_eq!(&options.name, "John Doe");
    assert!(options.primary);
    assert_eq!(options.phones.len(), 2);
}

#[test]
fn bind_should_deserialize_configuration_to_options() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
                ("name", "John Doe"),
                ("primary", "true"),
                ("phones:0", "+44 1234567"),
                ("phones:1", "+44 2345678"),
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build()
        .to_config();
    let mut options = ContactOptions::default();

    // act
    config.bind(&mut options);

    // assert
    assert_eq!(&options.name, "John Doe");
    assert!(options.primary);
    assert_eq!(options.phones.len(), 2);
}

#[test]
fn bind_at_should_deserialize_configuration_to_options() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
                ("contact:name", "John Doe"),
                ("contact:primary", "true"),
                ("contact:phones:0", "+44 1234567"),
                ("contact:phones:1", "+44 2345678"),
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build()
        .to_config();
    let mut options = ContactOptions::default();

    // act
    config.bind_at("contact", &mut options);

    // assert
    assert_eq!(&options.name, "John Doe");
    assert!(options.primary);
    assert_eq!(options.phones.len(), 2);
}

#[test]
fn get_value_should_deserialize_configuration_value() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
                ("name", "John Doe"),
                ("primary", "true"),
                ("phones:0", "+44 1234567"),
                ("phones:1", "+44 2345678"),
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build()
        .to_config();

    // act
    let primary: Option<bool> = config.get_value("primary").unwrap();

    // assert
    assert!(primary.unwrap());
}

#[test]
fn get_value_should_return_none_for_missing_configuration_value() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
                ("name", "John Doe"),
                ("phones:0", "+44 1234567"),
                ("phones:1", "+44 2345678"),
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build()
        .to_config();

    // act
    let primary: Option<bool> = config.get_value("primary").unwrap();

    // assert
    assert!(primary.is_none());
}

#[test]
fn get_value_or_default_should_return_default_value_for_missing_configuration_value() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
                ("name", "John Doe"),
                ("phones:0", "+44 1234567"),
                ("phones:1", "+44 2345678"),
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build()
        .to_config();

    // act
    let primary: bool = config.get_value_or_default("primary").unwrap();

    // assert
    assert!(!primary);
}

#[test]
fn deserialization_should_preserve_case_in_ini_file() {
    // arrange
    let path = PathBuf::new()
        .join(var("TEMP").unwrap())
        .join("test1.servicesettings.overrides.ini");
    let mut file = File::create(&path).unwrap();

    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Disabled=true\n").unwrap();
    file.write_all(b"AzureClusterClass:Compute$Disabled=false\n\n").unwrap();
    file.write_all(b"[FileCopySettings]\n").unwrap();
    file.write_all(b"UseSfpCopy = true\n").unwrap();
    file.write_all(b"AzureSDPRolloutPhase:Stage$UseSfpCopy=false\n").unwrap();
    file.write_all(b"AzureSDPRolloutPhase:Canary$UseSfpCopy=false\n\n").unwrap();
    file.write_all(b"[RequiredFiles]\n").unwrap();
    file.write_all(b"start.bat=1").unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_ini_file(&path)
        .build()
        .to_config();

    // act
    let mut settings = FileCopySettings::default();

    // act
    config.bind_at("FileCopySettings", &mut settings);    

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }

    assert!(settings.use_sfp_copy);
}