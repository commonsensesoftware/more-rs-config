use config::{prelude::*, Error};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tempfile::{tempdir, NamedTempFile};
use test_case::test_case;
use tokens::ChangeToken;

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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();
    let section = config.section("Data.Setting").section("DefaultConnection");

    // act
    let actual = section.get("Provider");

    // assert
    assert_eq!(actual, Some("SqlClient"));
}

#[test]
fn add_xml_file_should_fail_if_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.xml");

    // act
    let result = config::builder().add_xml_file(&path).build().load();

    // assert
    if let Err(error) = result {
        if matches!(error, Error::MissingFile(_)) {
            assert_eq!(
                &error.to_string(),
                r"The configuration file 'C:\fake\settings.xml' was not found, but is required."
            )
        } else {
            panic!("{:?}", error)
        }
    } else {
        panic!("No error occurred.")
    }
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let config = config::builder()
        .add_xml_file(file.path().is().optional())
        .build()
        .load()
        .unwrap();
    let section = config.section("Data.Setting").section("Inventory");

    // act
    let actual = section.get("Provider");

    // assert
    assert_eq!(actual, Some("MySql"));
}

#[test]
fn add_xml_file_should_succeed_if_optional_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.xml");

    // act
    let config = config::builder()
        .add_xml_file(&path.is().optional())
        .build()
        .load()
        .unwrap();

    // assert
    assert_eq!(config.sections().len(), 0);
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(config.get("Port"), Some("8008"));
    assert_eq!(config.get("Data:DefaultConnection:Provider"), Some("SqlClient"));
    assert_eq!(config.get("Data:Inventory:Provider"), Some("MySql"));
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(config.get("Port"), Some("8008"));
    assert_eq!(config.get("Data:DefaultConnection:Provider"), Some("SqlClient"));
    assert_eq!(config.get("Data:Inventory:Provider"), Some("MySql"));
}

#[test_case("Name" ; "with titlecase")]
#[test_case("name" ; "with lowercase")]
#[test_case("NAME" ; "with uppercase")]
fn name_attribute_should_contribute_to_prefix(attribute: &str) {
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(config.get("Data:DefaultConnection:Name"), Some("DefaultConnection"));
    assert_eq!(
        config.get("Data:DefaultConnection:ConnectionString"),
        Some("TestConnectionString")
    );
    assert_eq!(config.get("Data:DefaultConnection:Provider"), Some("SqlClient"));
    assert_eq!(config.get("Data:Inventory:Name"), Some("Inventory"));
    assert_eq!(config.get("Data:Inventory:Provider"), Some("MySql"));
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(config.get("Data:Name"), Some("Data"));
    assert_eq!(config.get("Data:DefaultConnection:Provider"), Some("SqlClient"));
    assert_eq!(config.get("Data:Inventory:Provider"), Some("MySql"));
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(config.get("DefaultConnection:0:Provider"), Some("SqlClient1"));
    assert_eq!(config.get("DefaultConnection:1:Provider"), Some("SqlClient2"));
}

#[test_case("DefaultConnection" ; "with titlecase")]
#[test_case("defaultconnection" ; "with lowercase")]
#[test_case("DEFAULTCONNECTION" ; "with uppercase")]
fn repeated_element_should_be_array_like(element: &str) {
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(config.get("DefaultConnection:0:Provider"), Some("SqlClient1"));
    assert_eq!(config.get("DefaultConnection:1:Provider"), Some("SqlClient2"));
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(config.get("DefaultConnection:Data1:Provider"), Some("SqlClient1"));
    assert_eq!(config.get("DefaultConnection:Data2:Provider"), Some("SqlClient2"));
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(
        config.get("DefaultConnection:0:ConnectionString:0"),
        Some("TestConnectionString1")
    );
    assert_eq!(
        config.get("DefaultConnection:0:ConnectionString:1"),
        Some("TestConnectionString2")
    );
    assert_eq!(
        config.get("DefaultConnection:1:ConnectionString:0"),
        Some("TestConnectionString3")
    );
    assert_eq!(
        config.get("DefaultConnection:1:ConnectionString:1"),
        Some("TestConnectionString4")
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // assert
    assert_eq!(
        config.get("DefaultConnection:0:ConnectionString"),
        Some("TestConnectionString1")
    );
    assert_eq!(
        config.get("DefaultConnection:1:ConnectionString"),
        Some("TestConnectionString2")
    );
    assert_eq!(
        config.get("DefaultConnection:2:ConnectionString"),
        Some("TestConnectionString3")
    );
    assert_eq!(config.get("DefaultConnection:0:Provider"), Some("SqlClient1"));
    assert_eq!(config.get("DefaultConnection:1:Provider"), Some("SqlClient2"));
    assert_eq!(config.get("DefaultConnection:2:Provider"), Some("SqlClient3"));
    assert_eq!(config.get("OtherValue:Value"), Some("MyValue"));
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // act
    let actual = config.get("Data:Inventory:Provider");

    // assert
    assert_eq!(actual, Some("SpecialStringWith<>"));
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    let config = config::builder().add_xml_file(file.path()).build().load().unwrap();

    // act
    let actual = config.get("Data:DefaultConnection:Provider");

    // assert
    assert_eq!(actual, Some("SqlClient"));
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let result = config::builder().add_xml_file(file.path()).build().load();

    // assert
    if let Err(error) = result {
        if matches!(error, Error::InvalidFile { .. }) {
            assert_eq!(&error.to_string(), "XML namespaces are not supported. (Data, Line: 2)")
        } else {
            panic!("{:#?}", error)
        }
    } else {
        panic!("No error occurred.")
    }
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
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(xml.to_string().as_bytes()).unwrap();

    // act
    let result = config::builder().add_xml_file(file.path()).build().load();

    // assert
    if let Err(error) = result {
        if matches!(error, Error::InvalidFile { .. }) {
            assert_eq!(
                &error.to_string(),
                "A duplicate key 'Data:DefaultConnection:ConnectionString' was found. (Data, Line: 7)"
            )
        } else {
            panic!("{:#?}", error)
        }
    } else {
        panic!("No error occurred.")
    }
}

#[test]
fn xml_file_should_reload_when_changed() {
    // arrange
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.xml");
    let mut file = File::create(&path).unwrap();
    let mut xml = concat!(
        "<Settings>\n",
        " <Connections>\n",
        "  <Connection Key=\"Default\" Retries=\"3\">\n",
        "   server=(locahost);database=test\n",
        "  </Connection>\n",
        " </Connections>\n",
        "</Settings>"
    );

    file.write_all(xml.to_string().as_bytes()).unwrap();
    drop(file);

    let root = config::builder().add_xml_file(&path.is().reloadable()).build();
    let mut config = root.load().unwrap();
    let section = config.section("Connections").section("Connection");
    let initial = section.get("Retries").unwrap_or_default().to_owned();

    drop(section);

    let token = config.reload_token();
    let state = Arc::new((Mutex::new(false), Condvar::new()));
    let _unused = token.register(
        Box::new(|s| {
            let data = s.unwrap();
            let (reloaded, event) = &*(data.downcast_ref::<(Mutex<bool>, Condvar)>().unwrap());
            *reloaded.lock().unwrap() = true;
            event.notify_one();
        }),
        Some(state.clone()),
    );

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
        reloaded = event.wait_timeout(reloaded, Duration::from_secs(1)).unwrap().0;
    }

    config = root.load().unwrap();

    // act
    let section = config.section("Connections").section("Connection");
    let current = section.get("Retries").unwrap_or_default();

    // assert
    assert_eq!(initial, "3");
    assert_eq!(current, "5");
}
