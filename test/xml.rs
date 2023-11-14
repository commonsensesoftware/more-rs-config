use config::{ext::*, *};
use std::env::temp_dir;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use test_case::test_case;

struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        if self.0.exists() {
            remove_file(&self.0).ok();
        }
    }
}

#[test]
fn add_xml_file_should_load_settings_from_file() {
    // arrange
    let xml = concat!(
        "<settings>\n",
        " <Data.Setting>\n",
        "  <DefaultConnection>\n",
        "   <Connection.String>Test.Connection.String</Connection.String>\n",
        "   <Provider>SqlClient</Provider>\n",
        "  </DefaultConnection>\n",
        "  <Inventory>\n",
        "   <ConnectionString>AnotherTestConnectionString</ConnectionString>\n",
        "   <Provider>MySql</Provider>\n",
        "  </Inventory>\n",
        " </Data.Setting>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_1.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();
    let section = config.section("Data.Setting").section("DefaultConnection");

    // act
    let result = section.get("Provider");

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    let value = result.unwrap();
    assert_eq!(*value, "SqlClient");
}

#[test]
fn add_xml_file_should_fail_if_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.xml");

    // act
    let result = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build();

    // assert
    if let Err(error) = result {
        if let ReloadError::Provider(errors) = error {
            assert_eq!(
                errors[0].1.message(),
                r"The configuration file 'C:\fake\settings.xml' was not found and is not optional."
            );
            return;
        }
    }

    panic!("Unexpected error.");
}

#[test]
fn add_optional_xml_file_should_load_settings_from_file() {
    // arrange
    let xml = concat!(
        "<settings>\n",
        " <Data.Setting>\n",
        "  <DefaultConnection>\n",
        "   <Connection.String>Test.Connection.String</Connection.String>\n",
        "   <Provider>SqlClient</Provider>\n",
        "  </DefaultConnection>\n",
        "  <Inventory>\n",
        "   <ConnectionString>AnotherTestConnectionString</ConnectionString>\n",
        "   <Provider>MySql</Provider>\n",
        "  </Inventory>\n",
        " </Data.Setting>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_2.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path.is().optional())
        .build()
        .unwrap();
    let section = config.section("Data.Setting").section("Inventory");

    // act
    let result = section.get("Provider");

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    let value = result.unwrap();
    assert_eq!(*value, "MySql");
}

#[test]
fn add_xml_file_should_succeed_if_optional_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.xml");

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path.is().optional())
        .build()
        .unwrap();

    // assert
    assert_eq!(config.children().len(), 0);
}

#[test]
fn add_xml_file_should_process_attributes() {
    // arrange
    let xml = concat!(
        "<settings Port=\"8008\">\n",
        " <Data>\n",
        "  <DefaultConnection\n",
        "   ConnectionString=\"TestConnectionString\"\n",
        "   Provider=\"SqlClient\"/>\n",
        "  <Inventory\n",
        "   ConnectionString=\"AnotherTestConnectionString\"\n",
        "   Provider=\"MySql\"/>\n",
        "  </Data>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_3.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(*config.get("Port").unwrap(), "8008");
    assert_eq!(
        *config.get("Data:DefaultConnection:Provider").unwrap(),
        "SqlClient"
    );
    assert_eq!(*config.get("Data:Inventory:Provider").unwrap(), "MySql");
}

#[test]
fn add_xml_file_should_mix_elements_and_attributes() {
    // arrange
    let xml = concat!(
        "<settings Port='8008'>\n",
        " <Data>\n",
        "  <DefaultConnection Provider='SqlClient'>\n",
        "   <ConnectionString>TestConnectionString</ConnectionString>\n",
        "  </DefaultConnection>\n",
        "  <Inventory ConnectionString='AnotherTestConnectionString'>\n",
        "   <Provider>MySql</Provider>\n",
        "  </Inventory>\n",
        " </Data>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_4.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(*config.get("Port").unwrap(), "8008");
    assert_eq!(
        *config.get("Data:DefaultConnection:Provider").unwrap(),
        "SqlClient"
    );
    assert_eq!(*config.get("Data:Inventory:Provider").unwrap(), "MySql");
}

#[test_case("test_settings_5.1.xml", "Name" ; "with titlecase")]
#[test_case("test_settings_5.2.xml", "name" ; "with lowercase")]
#[test_case("test_settings_5.3.xml", "NAME" ; "with uppercase")]
fn name_attribute_should_contribute_to_prefix(filename: &str, attribute: &str) {
    // arrange
    let xml = &[
        "<settings>\n",
        &format!(" <Data {}='DefaultConnection'>\n", attribute),
        "  <ConnectionString>TestConnectionString</ConnectionString>\n",
        "  <Provider>SqlClient</Provider>\n",
        " </Data>\n",
        &format!(" <Data {}='Inventory'>\n", attribute),
        "  <ConnectionString>AnotherTestConnectionString</ConnectionString>\n",
        "  <Provider>MySql</Provider>\n",
        " </Data>\n",
        "</settings>",
    ]
    .join("");
    let path = temp_dir().join(filename);
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(
        *config.get("Data:DefaultConnection:Name").unwrap(),
        "DefaultConnection"
    );
    assert_eq!(
        *config
            .get("Data:DefaultConnection:ConnectionString")
            .unwrap(),
        "TestConnectionString"
    );
    assert_eq!(
        *config.get("Data:DefaultConnection:Provider").unwrap(),
        "SqlClient"
    );
    assert_eq!(*config.get("Data:Inventory:Name").unwrap(), "Inventory");
    assert_eq!(*config.get("Data:Inventory:Provider").unwrap(), "MySql");
}

#[test]
fn root_element_name_attribute_should_contribute_to_prefix() {
    // arrange
    let xml = concat!(
        "<settings Name='Data'>\n",
        " <DefaultConnection>\n",
        "  <ConnectionString>TestConnectionString</ConnectionString>\n",
        "  <Provider>SqlClient</Provider>\n",
        "  </DefaultConnection>\n",
        " <Inventory>\n",
        "  <ConnectionString>AnotherTestConnectionString</ConnectionString>\n",
        "  <Provider>MySql</Provider>\n",
        " </Inventory>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_6.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(*config.get("Data:Name").unwrap(), "Data");
    assert_eq!(
        *config.get("Data:DefaultConnection:Provider").unwrap(),
        "SqlClient"
    );
    assert_eq!(*config.get("Data:Inventory:Provider").unwrap(), "MySql");
}

#[test]
fn numeric_name_attribute_should_be_array_like() {
    // arrange
    let xml = concat!(
        "<settings>\n",
        " <DefaultConnection Name='0'>\n",
        "  <ConnectionString>TestConnectionString1</ConnectionString>\n",
        "  <Provider>SqlClient1</Provider>\n",
        " </DefaultConnection>\n",
        " <DefaultConnection Name='1'>\n",
        "  <ConnectionString>TestConnectionString2</ConnectionString>\n",
        "  <Provider>SqlClient2</Provider>\n",
        " </DefaultConnection>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_7.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(
        *config.get("DefaultConnection:0:Provider").unwrap(),
        "SqlClient1"
    );
    assert_eq!(
        *config.get("DefaultConnection:1:Provider").unwrap(),
        "SqlClient2"
    );
}

#[test_case("test_settings_8.1.xml", "DefaultConnection" ; "with titlecase")]
#[test_case("test_settings_8.2.xml", "defaultconnection" ; "with lowercase")]
#[test_case("test_settings_8.3.xml", "DEFAULTCONNECTION" ; "with uppercase")]
fn repeated_element_should_be_array_like(filename: &str, element: &str) {
    // arrange
    let xml = &[
        "<settings>\n",
        " <DefaultConnection>\n",
        "  <ConnectionString>TestConnectionString1</ConnectionString>\n",
        "  <Provider>SqlClient1</Provider>\n",
        " </DefaultConnection>\n",
        &format!(" <{}>\n", element),
        "  <ConnectionString>TestConnectionString2</ConnectionString>\n",
        "  <Provider>SqlClient2</Provider>\n",
        &format!(" </{}>\n", element),
        "</settings>",
    ]
    .join("");
    let path = temp_dir().join(filename);
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(
        *config.get("DefaultConnection:0:Provider").unwrap(),
        "SqlClient1"
    );
    assert_eq!(
        *config.get("DefaultConnection:1:Provider").unwrap(),
        "SqlClient2"
    );
}

#[test]
fn repeated_element_with_different_name_attribute_should_have_different_prefix() {
    // arrange
    let xml = concat!(
        "<settings>\n",
        " <DefaultConnection Name='Data1'>\n",
        "  <ConnectionString>TestConnectionString1</ConnectionString>\n",
        "  <Provider>SqlClient1</Provider>\n",
        " </DefaultConnection>\n",
        " <DefaultConnection Name='Data2'>\n",
        "  <ConnectionString>TestConnectionString2</ConnectionString>\n",
        "  <Provider>SqlClient2</Provider>\n",
        " </DefaultConnection>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_9.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(
        *config.get("DefaultConnection:Data1:Provider").unwrap(),
        "SqlClient1"
    );
    assert_eq!(
        *config.get("DefaultConnection:Data2:Provider").unwrap(),
        "SqlClient2"
    );
}

#[test]
fn nested_repeated_element_should_be_array_like() {
    // arrange
    let xml = concat!(
        "<settings>\n",
        " <DefaultConnection>\n",
        "  <ConnectionString>TestConnectionString1</ConnectionString>\n",
        "  <ConnectionString>TestConnectionString2</ConnectionString>\n",
        " </DefaultConnection>\n",
        " <DefaultConnection>\n",
        "  <ConnectionString>TestConnectionString3</ConnectionString>\n",
        "  <ConnectionString>TestConnectionString4</ConnectionString>\n",
        " </DefaultConnection>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_10.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(
        *config
            .get("DefaultConnection:0:ConnectionString:0")
            .unwrap(),
        "TestConnectionString1"
    );
    assert_eq!(
        *config
            .get("DefaultConnection:0:ConnectionString:1")
            .unwrap(),
        "TestConnectionString2"
    );
    assert_eq!(
        *config
            .get("DefaultConnection:1:ConnectionString:0")
            .unwrap(),
        "TestConnectionString3"
    );
    assert_eq!(
        *config
            .get("DefaultConnection:1:ConnectionString:1")
            .unwrap(),
        "TestConnectionString4"
    );
}

#[test]
fn mixed_repeated_element_should_be_array_like() {
    // arrange
    let xml = concat!(
        "<settings>\n",
        " <DefaultConnection>\n",
        "  <ConnectionString>TestConnectionString1</ConnectionString>\n",
        "  <Provider>SqlClient1</Provider>\n",
        " </DefaultConnection>\n",
        " <DefaultConnection>\n",
        "  <ConnectionString>TestConnectionString2</ConnectionString>\n",
        "  <Provider>SqlClient2</Provider>\n",
        " </DefaultConnection>\n",
        " <OtherValue>\n",
        "  <Value>MyValue</Value>\n",
        " </OtherValue>\n",
        " <DefaultConnection>\n",
        "  <ConnectionString>TestConnectionString3</ConnectionString>\n",
        "  <Provider>SqlClient3</Provider>\n",
        " </DefaultConnection>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_11.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(
        *config.get("DefaultConnection:0:ConnectionString").unwrap(),
        "TestConnectionString1"
    );
    assert_eq!(
        *config.get("DefaultConnection:1:ConnectionString").unwrap(),
        "TestConnectionString2"
    );
    assert_eq!(
        *config.get("DefaultConnection:2:ConnectionString").unwrap(),
        "TestConnectionString3"
    );
    assert_eq!(
        *config.get("DefaultConnection:0:Provider").unwrap(),
        "SqlClient1"
    );
    assert_eq!(
        *config.get("DefaultConnection:1:Provider").unwrap(),
        "SqlClient2"
    );
    assert_eq!(
        *config.get("DefaultConnection:2:Provider").unwrap(),
        "SqlClient3"
    );
    assert_eq!(*config.get("OtherValue:Value").unwrap(), "MyValue");
}

#[test]
fn config_values_should_process_cdata() {
    // arrange
    let xml = concat!(
        "<settings>\n",
        " <Data>\n",
        "  <Inventory>\n",
        "   <Provider><![CDATA[SpecialStringWith<>]]></Provider>\n",
        "  </Inventory>\n",
        " </Data>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_12.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // act
    let value = config.get("Data:Inventory:Provider");

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(*value.unwrap(), "SpecialStringWith<>");
}

#[test]
fn xml_declaration_and_processing_instructions_should_be_ignored() {
    // arrange
    let xml = concat!(
        "<?xml version='1.0' encoding='UTF-8'?>\n",
        "<?xml-stylesheet type='text/xsl' href='style1.xsl'?>\n",
        "<settings>\n",
        " <?xml-stylesheet type='text/xsl' href='style2.xsl'?>\n",
        " <Data>\n",
        "  <DefaultConnection>\n",
        "   <ConnectionString>TestConnectionString</ConnectionString>\n",
        "   <Provider>SqlClient</Provider>\n",
        "  </DefaultConnection>\n",
        "  <Inventory>\n",
        "   <ConnectionString>AnotherTestConnectionString</ConnectionString>\n",
        "   <Provider>MySql</Provider>\n",
        "  </Inventory>\n",
        " </Data>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_13.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build()
        .unwrap();

    // act
    let value = config.get("Data:DefaultConnection:Provider");

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(*value.unwrap(), "SqlClient");
}

#[test]
fn load_should_fail_when_xml_namespace_is_encountered() {
    // arrange
    let xml = concat!(
        "<settings xmlns:MyNamespace='http://w3c.org/test/mynamespace'>\n",
        " <MyNamespace:Data>\n",
        "  <DefaultConnection>\n",
        "   <ConnectionString>TestConnectionString</ConnectionString>\n",
        "   <Provider>SqlClient</Provider>\n",
        "  </DefaultConnection>\n",
        "  <Inventory>\n",
        "   <ConnectionString>AnotherTestConnectionString</ConnectionString>\n",
        "   <Provider>MySql</Provider>\n",
        "  </Inventory>\n",
        " </MyNamespace:Data>\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_14.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let _file = TempFile(path.clone());

    // act
    let result = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build();

    // assert
    if let Err(error) = result {
        if let ReloadError::Provider(errors) = error {
            assert_eq!(errors[0].1.message(), "XML namespaces are not supported. (Data, Line: 2)");
            return;
        }
    }

    panic!("Unexpected error.");
}

#[test]
fn load_should_fail_when_key_is_duplicated() {
    // arrange
    let xml = concat!(
        "<settings>\n",
        " <Data>\n",
        "  <DefaultConnection>\n",
        "   <ConnectionString>TestConnectionString</ConnectionString>\n",
        "  </DefaultConnection>\n",
        " </Data>\n",
        " <Data Name='DefaultConnection' ConnectionString='NewConnectionString' />\n",
        "</settings>"
    );
    let path = temp_dir().join("test_settings_15.xml");
    let mut file = File::create(&path).unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let _file = TempFile(path.clone());

    // act
    let result = DefaultConfigurationBuilder::new()
        .add_xml_file(&path)
        .build();

    // assert
    if let Err(error) = result {
        if let ReloadError::Provider(errors) = error {
            let message = errors[0].1.message();
            
            // this can vary because the key processing doesn't guarantee stable ordering
            // the net effect is the same. it's just a matter of which one is hit first
            assert!((message == "A duplicate key 'Data:DefaultConnection:ConnectionString' was found. (Data, Line: 7)") ||
                    (message == "A duplicate key 'Data:DefaultConnection:ConnectionString' was found. (ConnectionString, Line: 4)"));
            return;
        }
    }

    panic!("Unexpected error.");
}

#[test]
fn xml_file_should_reload_when_changed() {
    // arrange
    let path = temp_dir().join("reload_settings_1.xml");
    let mut xml = concat!(
        "<Settings>\n",
        " <Connections>\n",
        "  <Connection Key=\"Default\" Retries=\"3\">\n",
        "   server=(locahost);database=test\n",
        "  </Connection>\n",
        " </Connections>\n",
        "</Settings>"
    );

    let mut file = File::create(&path).unwrap();
    file.write_all(xml.to_string().as_bytes()).unwrap();
    drop(file);

    let config = DefaultConfigurationBuilder::new()
        .add_xml_file(&path.is().reloadable())
        .build()
        .unwrap();
    let section = config.section("Connections").section("Connection");
    let initial = section.get("Retries").unwrap_or_default().into_owned();

    drop(section);

    let token = config.reload_token();
    let state = Arc::new((Mutex::new(false), Condvar::new()));
    let other_state = Arc::clone(&state);
    let _unused = token.register(Box::new(move || {
        let (reloaded, event) = &*other_state;
        *reloaded.lock().unwrap() = true;
        event.notify_one();
    }));

    xml = concat!(
        "<Settings>\n",
        " <Connections>\n",
        "  <Connection Key=\"Default\" Retries=\"5\">\n",
        "   server=(locahost);database=test\n",
        "  </Connection>\n",
        " </Connections>\n",
        "</Settings>"
    );

    file = File::create(&path).unwrap();
    file.write_all(xml.to_string().as_bytes()).unwrap();
    drop(file);

    let (mutex, event) = &*state;
    let mut reloaded = mutex.lock().unwrap();

    while !*reloaded {
        reloaded = event
            .wait_timeout(reloaded, Duration::from_secs(1))
            .unwrap()
            .0;
    }

    // act
    let section = config.section("Connections").section("Connection");
    let current = section.get("Retries").unwrap_or_default();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }

    assert_eq!(&initial, "3");
    assert_eq!(*current, "5");
}
