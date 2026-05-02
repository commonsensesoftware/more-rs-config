{{#include links.md}}

# XML Configuration Provider

>These features are only available if the **xml** feature is activated

The [xml::Provider] supports loading configuration from a `*.xml` file.

The following code adds several configuration providers, including a couple of `*.xml` files:

```rust
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let name = std::env::var("ENVIRONMENT").or_else("production");
    let config = config::builder()
        .add_xml_file("MyXmlConfig.xml".is().optional())
        .add_xml_file(format!("MyXmlConfig.{name}.xml").is().optional())
        .add_env_vars()
        .add_command_line()
        .build()?;

    Ok(())
}
```

The XML configuration files have a few special rules that are different from other
[configuration providers](abstractions.md#configuration-provider):

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

println!("MyKey value: {my_key_value}\n\
          Title: {title}\n\
          Name: {name}\n\
          Default Log Level: {default_log_level}");
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
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = config::builder().add_xml_file("MyXmlConfig2.xml").build()?;
    let val00 = config.get("section:section0:key:key0").unwrap();
    let val01 = config.get("section:section0:key:key1").unwrap();
    let val10 = config.get("section:section1:key:key0").unwrap();
    let val11 = config.get("section:section1:key:key1").unwrap();

    println!("section:section0:key:key0 value: {val00}\n\
              section:section0:key:key1 value: {val01}\n\
              section:section1:key:key0 value: {val10}\n\
              section:section1:key:key1 value: {val11}");

    Ok(())
}
```

If the `name` attribute were not used, then the elements would be treated as _array-like_:

- `section:0:key:0`
- `section:0:key:1`
- `section:1:key:0`
- `section:1:key:1`

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

- `key:attribute`
- `section:key:attribute`
