# Abstractions

The configuration framework contains a common set of traits and behaviors for numerous scenarios.

## Configuration

The `Configuration` trait is the pinnacle of the entire framework. It defines the behaviors to retrieve a configured value or iterate over all key-value pairs, access or traverse child sections, and react to a reload triggered by the underlying configuration source.

```rust
pub trait Configuration {
    fn get(&self, key: &str) -> Option<Cow<String>>;
    fn section(&self, key: &str) -> Box<dyn ConfigurationSection>;
    fn children(&self) -> Vec<Box<dyn ConfigurationSection>>;
    fn reload_token(&self) -> Box<dyn ChangeToken>;
    fn as_section(&self) -> Option<&dyn ConfigurationSection>;
    fn iter(&self) -> Box<dyn Iterator<Item = (String, String)>>;
    fn iter_relative(
        &self,
        make_paths_relative: bool,
    ) -> Box<dyn Iterator<Item = (String, String)>>;
}
```

## Configuration Section

Hierarchical configurations are divided into _sections_. A configurations section is itself a nested `Configuration`. A configuration section also has its own key and, possibly, a value. A configuration section which does not have a value will always yield an empty string.

```rust
pub trait ConfigurationSection:
    Configuration
    + AsRef<dyn Configuration>
    + Borrow<dyn Configuration>
    + Deref<Target = dyn Configuration>
{
    fn key(&self) -> &str;
    fn path(&self) -> &str;
    fn value(&self) -> Cow<String>;
}
```

## Configuration Root

Every configuration has a single root. The root configuration knows about all of the associated `ConfigurationProvider` instances and can reload the entire configuration.

```rust
pub trait ConfigurationRoot:
    Configuration
    + AsRef<dyn Configuration>
    + Borrow<dyn Configuration>
    + Deref<Target = dyn Configuration>
    + Debug
{
    fn reload(&mut self) -> ReloadResult;
    fn providers(&self) -> Box<dyn ConfigurationProviderIterator + '_>;
    fn as_config(&self) -> Box<dyn Configuration>;
}
```

# Configuration Provider

A configuration provider is responsible for loading configuration from a source. A configuration provider might support automatic reloading and can advertise when a reload has occurred via a reload `ChangeToken`. A configuration value uses _Copy-On-Write_ semantics, which allows providers that cannot reload (ex: in-memory) to provided borrowed values, while providers that do reload  can provide owned values (ex: file-based).

```rust
pub trait ConfigurationProvider {
    fn name(&self) -> &str;
    fn get(&self, key: &str) -> Option<Cow<String>>;
    fn reload_token(&self) -> Box<dyn ChangeToken>;
    fn load(&mut self) -> LoadResult;
    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>);
}
```

# Configuration Source

A configuration source provides an abstraction over a source for configuration such as a file. The source accepts all of the information required to setup a provider and then constructs it when the configuration is built.

```rust
pub trait ConfigurationSource {
    fn build(
        &self,
        builder: &dyn ConfigurationBuilder
    ) -> Box<dyn ConfigurationProvider>;
}
```

## Configuration Builder

A configuration builder accumulates one or more configuration sources and then builds a `ConfigurationRoot`. The configuration is immediately reloaded so that it is ready to use.

```rust
pub trait ConfigurationBuilder {
    fn properties(&self) -> &HashMap<String, Box<dyn Any>>;
    fn sources(&self) -> &[Box<dyn ConfigurationSource>];
    fn add(&mut self, source: Box<dyn ConfigurationSource>);
    fn build(&self) -> Result<Box<dyn ConfigurationRoot>, ReloadError>;
}
```