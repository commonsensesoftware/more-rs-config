use crate::{
    util::*, ConfigurationBuilder, ConfigurationPath, ConfigurationProvider, ConfigurationSource,
    FileSource, LoadError, LoadResult, Value,
};
use serde_json::{map::Map, Value as JsonValue};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use tokens::{ChangeToken, FileChangeToken, SharedChangeToken, SingleChangeToken, Subscription};

#[derive(Default)]
struct JsonVisitor {
    data: HashMap<String, (String, Value)>,
    paths: Vec<String>,
}

impl JsonVisitor {
    fn visit(mut self, root: &Map<String, JsonValue>) -> HashMap<String, (String, Value)> {
        self.visit_element(root);
        self.data.shrink_to_fit();
        self.data
    }

    fn visit_element(&mut self, element: &Map<String, JsonValue>) {
        if element.is_empty() {
            if let Some(key) = self.paths.last() {
                self.data
                    .insert(key.to_uppercase(), (to_pascal_case(key), String::new().into()));
            }
        } else {
            for (name, value) in element {
                self.enter_context(to_pascal_case(name));
                self.visit_value(value);
                self.exit_context();
            }
        }
    }

    fn visit_value(&mut self, value: &JsonValue) {
        match value {
            JsonValue::Object(ref element) => self.visit_element(element),
            JsonValue::Array(array) => {
                for (index, element) in array.iter().enumerate() {
                    self.enter_context(index.to_string());
                    self.visit_value(element);
                    self.exit_context();
                }
            }
            JsonValue::Bool(value) => self.add_value(value),
            JsonValue::Null => self.add_value(String::new()),
            JsonValue::Number(value) => self.add_value(value),
            JsonValue::String(value) => self.add_value(value),
        }
    }

    fn add_value<T: ToString>(&mut self, value: T) {
        let key = self.paths.last().unwrap().to_string();
        self.data
            .insert(key.to_uppercase(), (key, value.to_string().into()));
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
    data: RwLock<HashMap<String, (String, Value)>>,
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

    fn load(&self, reload: bool) -> LoadResult {
        if !self.file.path.is_file() {
            if self.file.optional || reload {
                let mut data = self.data.write().unwrap();
                if !data.is_empty() {
                    *data = HashMap::with_capacity(0);
                }

                return Ok(());
            } else {
                return Err(LoadError::File {
                    message: format!(
                        "The configuration file '{}' was not found and is not optional.",
                        self.file.path.display()
                    ),
                    path: self.file.path.clone(),
                });
            }
        }

        // REF: https://docs.serde.rs/serde_json/de/fn.from_reader.html
        let content = fs::read(&self.file.path).unwrap();
        let json: JsonValue = serde_json::from_slice(&content).unwrap();

        if let Some(root) = json.as_object() {
            let visitor = JsonVisitor::default();
            let data = visitor.visit(root);
            *self.data.write().unwrap() = data;
        } else if reload {
            *self.data.write().unwrap() = HashMap::with_capacity(0);
        } else {
            return Err(LoadError::File {
                message: format!(
                    "Top-level JSON element must be an object. Instead, '{}' was found.",
                    match json {
                        JsonValue::Array(_) => "array",
                        JsonValue::Bool(_) => "Boolean",
                        JsonValue::Null => "null",
                        JsonValue::Number(_) => "number",
                        JsonValue::String(_) => "string",
                        _ => unreachable!(),
                    }
                ),
                path: self.file.path.clone(),
            });
        }

        let previous = std::mem::replace(
            &mut *self.token.write().unwrap(),
            SharedChangeToken::default(),
        );

        previous.notify();
        Ok(())
    }

    fn get(&self, key: &str) -> Option<Value> {
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
    _subscription: Option<Box<dyn Subscription>>,
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
        let subscription: Option<Box<dyn Subscription>> = if inner.file.reload_on_change {
            Some(Box::new(tokens::on_change(
                move || FileChangeToken::new(path.clone()),
                |state| {
                    let provider = state.unwrap();
                    std::thread::sleep(provider.file.reload_delay);
                    provider.load(true).ok();
                },
                Some(inner.clone())
            )))
        } else {
            None
        };

        Self {
            inner,
            _subscription: subscription,
        }
    }
}

impl ConfigurationProvider for JsonConfigurationProvider {
    fn get(&self, key: &str) -> Option<Value> {
        self.inner.get(key)
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        self.inner.reload_token()
    }

    fn load(&mut self) -> LoadResult {
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
