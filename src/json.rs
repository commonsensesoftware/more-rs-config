#![allow(dyn_drop)]

use crate::{
    util::*, ConfigurationBuilder, ConfigurationPath, ConfigurationProvider, ConfigurationSource,
    FileSource,
};
use serde_json::{map::Map, Value};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use tokens::{ChangeToken, FileChangeToken, SharedChangeToken, SingleChangeToken};

#[derive(Default)]
struct JsonVisitor {
    data: HashMap<String, (String, String)>,
    paths: Vec<String>,
}

impl JsonVisitor {
    fn visit(mut self, root: &Map<String, Value>) -> HashMap<String, (String, String)> {
        self.visit_element(root);
        self.data.shrink_to_fit();
        self.data
    }

    fn visit_element(&mut self, element: &Map<String, Value>) {
        if element.is_empty() {
            if let Some(key) = self.paths.last() {
                self.data
                    .insert(key.to_uppercase(), (to_pascal_case(key), String::new()));
            }
        } else {
            for (name, value) in element {
                self.enter_context(to_pascal_case(name));
                self.visit_value(value);
                self.exit_context();
            }
        }
    }

    fn visit_value(&mut self, value: &Value) {
        match value {
            Value::Object(ref element) => self.visit_element(element),
            Value::Array(array) => {
                for (index, element) in array.iter().enumerate() {
                    self.enter_context(index.to_string());
                    self.visit_value(element);
                    self.exit_context();
                }
            }
            Value::Bool(value) => self.add_value(value),
            Value::Null => self.add_value(String::new()),
            Value::Number(value) => self.add_value(value),
            Value::String(value) => self.add_value(value),
        }
    }

    fn add_value<T: ToString>(&mut self, value: T) {
        let key = self.paths.last().unwrap().to_string();
        self.data
            .insert(key.to_uppercase(), (key, value.to_string()));
    }

    fn enter_context(&mut self, context: String) {
        if self.paths.is_empty() {
            self.paths.push(context);
            return;
        }

        let path = ConfigurationPath::combine(&[&self.paths[self.paths.len() - 1], &context]);
        self.paths.push(path);
    }

    fn exit_context(&mut self) {
        self.paths.pop();
    }
}

struct InnerProvider {
    file: FileSource,
    data: RwLock<HashMap<String, (String, String)>>,
    token: RwLock<SharedChangeToken<SingleChangeToken>>,
}

impl InnerProvider {
    fn new(file: FileSource) -> Self {
        Self {
            file,
            data: RwLock::new(HashMap::with_capacity(0)),
            token: Default::default(),
        }
    }

    fn load(&self, reload: bool) {
        if !self.file.path.is_file() {
            if self.file.optional || reload {
                let mut data = self.data.write().unwrap();
                if !data.is_empty() {
                    *data = HashMap::with_capacity(0);
                }

                return;
            } else {
                panic!(
                    "The configuration file '{}' was not found and is not optional.",
                    self.file.path.display()
                );
            }
        }

        // REF: https://docs.serde.rs/serde_json/de/fn.from_reader.html
        let content = fs::read(&self.file.path).unwrap();
        let json: Value = serde_json::from_slice(&content).unwrap();

        if let Some(root) = json.as_object() {
            let visitor = JsonVisitor::default();
            let data = visitor.visit(root);
            *self.data.write().unwrap() = data;
        } else if reload {
            *self.data.write().unwrap() = HashMap::with_capacity(0);
        } else {
            panic!(
                "Top-level JSON element must be an object. Instead, '{}' was found.",
                match json {
                    Value::Array(_) => "array",
                    Value::Bool(_) => "Boolean",
                    Value::Null => "null",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    _ => unreachable!(),
                }
            );
        }

        let previous = std::mem::replace(
            &mut *self.token.write().unwrap(),
            SharedChangeToken::default(),
        );

        previous.notify();
    }

    fn get(&self, key: &str) -> Option<String> {
        self.data
            .read()
            .unwrap()
            .get(&key.to_uppercase())
            .map(|t| t.1.clone())
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(self.token.read().unwrap().clone())
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        let data = self.data.read().unwrap();
        accumulate_child_keys(&data, earlier_keys, parent_path)
    }
}

/// Represents a [configuration provider](trait.ConfigurationProvider.html) for JSON files.
pub struct JsonConfigurationProvider {
    inner: Arc<InnerProvider>,
    _registration: Option<Box<dyn Drop>>,
}

impl JsonConfigurationProvider {
    /// Initializes a new JSON file configuration provider.
    ///
    /// # Arguments
    ///
    /// * `file` - The [JSON file](struct.FileSource.html) information
    pub fn new(file: FileSource) -> Self {
        let path = file.path.clone();
        let inner = Arc::new(InnerProvider::new(file));
        let registration: Option<Box<dyn Drop>> = if inner.file.reload_on_change {
            let other = inner.clone();

            Some(Box::new(tokens::on_change(
                move || FileChangeToken::new(path.clone()),
                move || {
                    std::thread::sleep(other.file.reload_delay);
                    other.load(true);
                },
            )))
        } else {
            None
        };

        Self {
            inner,
            _registration: registration,
        }
    }
}

impl ConfigurationProvider for JsonConfigurationProvider {
    fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key)
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        self.inner.reload_token()
    }

    fn load(&mut self) {
        self.inner.load(false)
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        self.inner.child_keys(earlier_keys, parent_path)
    }
}

/// Represents a [configuration source](trait.ConfigurationSource.html) for JSON files.
pub struct JsonConfigurationSource {
    file: FileSource,
}

impl JsonConfigurationSource {
    /// Initializes a new JSON file configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The [JSON file](struct.FileSource.html) information
    pub fn new(file: FileSource) -> Self {
        Self { file }
    }
}

impl ConfigurationSource for JsonConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(JsonConfigurationProvider::new(self.file.clone()))
    }
}

pub mod ext {

    use super::*;

    /// Defines extension methods for the [ConfigurationBuilder](trait.ConfigurationBuilder.html) trait.
    pub trait JsonConfigurationExtensions {
        /// Adds a JSON file as a configuration source.
        ///
        /// # Arguments
        ///
        /// * `file` - The [JSON file](struct.FileSource.html) information
        fn add_json_file<T: Into<FileSource>>(&mut self, file: T) -> &mut Self;
    }

    impl JsonConfigurationExtensions for dyn ConfigurationBuilder {
        fn add_json_file<T: Into<FileSource>>(&mut self, file: T) -> &mut Self {
            self.add(Box::new(JsonConfigurationSource::new(file.into())));
            self
        }
    }

    impl<T: ConfigurationBuilder> JsonConfigurationExtensions for T {
        fn add_json_file<F: Into<FileSource>>(&mut self, file: F) -> &mut Self {
            self.add(Box::new(JsonConfigurationSource::new(file.into())));
            self
        }
    }
}
