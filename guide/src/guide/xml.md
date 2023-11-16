# XML Configuration Provider

>These features are only available if the **xml** feature is activated

The `XmlConfigurationProvider` supports loading configuration from a `*.xml` file.

The following code adds several configuration providers, including a couple of `*.xml` files:

```rust
fn main() {
    let name = env::var("ENVIRONMENT").or_else("production");
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file("MyXmlConfig.xml".is().optional())
        .add_xml_file(format!("MyXmlConfig.{}.xml", name).is().optional())
        .add_env_vars()
        .add_command_line()
        .build()
        .unwrap();
}
```

The XML configuration files have a few special rules that are different from other [configuration providers](abstractions.md#configuration-provider):

1. XML namespaces are not supported on elements or attributes
2. The `Name` attribute (case-insensitive) is considered as a surrogate key in lieu of the element it is applied to
3. Duplicate key-value combinations are ambiguous and not allowed
4. Repeating elements with different values are considered _array-like_

Consider the following configuration file:

```xml
<?xml version="1.0" encoding="utf-8" ?>
<configuration>
  <MyKey>MyXMLFile Value</MyKey>
  <Position>
    <Title>Title from  MyXMLFile</Title>
    <Name>Name from MyXMLFile</Name>
  </Position>
  <Logging>
    <LogLevel>
      <Default>Information</Default>
      <App>Warning</App>
    </LogLevel>
  </Logging>
</configuration>
```

The following code displays several of the preceding configuration settings:

```rust
let my_key_value = config.get("MyKey").unwrap();
let title = config.get("Position:Title").unwrap();
let name = config.section("Position").get("Name").unwrap();
let default_log_level = config.get("Logging:LogLevel:Default").unwrap();

println!("MyKey value: {}\n\
          Title: {}\n\
          Name: {}\n\
          Default Log Level: {}",
          my_key_value,
          title,
          name,
          default_log_level);
```

Repeating elements that use the same element name work if the `name` attribute is used to distinguish the elements:

```xml
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <section name="section0">
    <key name="key0">value 00</key>
    <key name="key1">value 01</key>
  </section>
  <section name="section1">
    <key name="key0">value 10</key>
    <key name="key1">value 11</key>
  </section>
</configuration>
```

The following code reads the previous configuration file and displays the keys and values:

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_xml_file("MyXmlConfig2.xml")
        .build()
        .unwrap();

    let val00 = config.get("section:section0:key:key0");
    let val01 = config.get("section:section0:key:key1");
    let val10 = config.get("section:section1:key:key0");
    let val11 = config.get("section:section1:key:key1");

    println!("section:section0:key:key0 value: {}\n\
              section:section0:key:key1 value: {}\n\
              section:section1:key:key0 value: {}\n\
              section:section1:key:key1 value: {}",
              val00
              val01
              val10
              val11);
}
```

If the `name` attribute were not used, then the elements would be treated as _array-like_:

- section:0:key:0
- section:0:key:1
- section:1:key:0
- section:1:key:1

Attributes can also be used to supply values:

```xml
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <key attribute="value" />
  <section>
    <key attribute="value" />
  </section>
</configuration>
```

The previous configuration file loads the following keys with value of `value`:

- key:attribute
- section:key:attribute
