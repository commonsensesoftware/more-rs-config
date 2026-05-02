{{#include links.md}}

# Abstractions

The configuration framework contains a common set of traits and behaviors for numerous scenarios.

## Configuration

The [Configuration] struct is the pinnacle of the entire framework. It defines the behaviors to retrieve a configured
value or iterate over all key-value pairs, access or traverse child sections, and react to a reload triggered by the
underlying configuration source.

```rust
pub struct Configuration {
    pub fn get(&self, key: &str) -> Option<&str>;
    pub fn section(&self, key: impl Into<String>) -> Section<'_>;
    pub fn sections(&self) -> Vec<Section<'_>>;
    pub fn reload_token(&self) -> impl ChangeToken;
}
```

The entire configuration can be enumerated as tuples of key/value pairs similar to a `HashMap`.

## Configuration Section

Hierarchical configurations are divided into _sections_. A configuration [Section] has its own key and, possibly, a
value. A configuration section which does not have a value will always yield an empty string.

```rust
pub struct Section<'a> {
    pub fn key(&self) -> &str;
    pub fn value(&self) -> &str;
    pub fn path(&self) -> &str;
    pub fn exists(&self) -> bool;
    pub fn get(&self, key: &str) -> Option<&str>;
    pub fn section(&self, key: &str) -> Section<'a>;
    pub fn sections(&self) -> Vec<Section<'a>>
}
```

A configuration section can also be enumerated as tuples of key/value pairs similar to a `HashMap`.

# Configuration Provider

A configuration [Provider] is responsible for loading configuration key/value pairs as a collection of [Settings].
A provider might support automatic reloading and can advertise when a reload has occurred via a reload [ChangeToken].

```rust
pub trait Provider {
    fn name(&self) -> &str;
    fn reload_token(&self) -> Box<dyn ChangeToken>;
    fn load(&self, settings: &mut Settings) -> config::Result;
}
```

## Configuration Builder

A configuration builder accumulates one or more configuration providers and then builds a [Configuration].

```rust
pub struct Builder {
    pub fn providers(&self) -> impl Iterator<Item = &dyn Provider>;
    pub fn add(&mut self, provider: impl Provider + 'static);
    pub fn build(&self) -> config::Result<Configuration>;
}
```