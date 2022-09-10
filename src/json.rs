use crate::{
    util::accumulate_child_keys, ConfigurationBuilder, ConfigurationPath, ConfigurationProvider,
    ConfigurationSource,
};
use serde_json::{map::Map, Value};
use std::{collections::HashMap, fs, path::{Path, PathBuf}};

fn to_pascal_case<T: AsRef<str>>(text: T) -> String {
    let input = text.as_ref();
    let mut pascal_case = String::with_capacity(input.len());
    let mut chars = input.chars();

    if let Some(first) = chars.next() {
        pascal_case.push(first.to_ascii_uppercase());

        for ch in chars {
            pascal_case.push(ch);
        }
    }

    pascal_case
}

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
        self.data.insert(key.to_uppercase(), (key, value.to_string()));
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

/// Represents a [configuration provider](trait.ConfigurationProvider.html) for JSON files.
pub struct JsonConfigurationProvider {
    path: PathBuf,
    optional: bool,
    data: HashMap<String, (String, String)>,
}

impl JsonConfigurationProvider {
    /// Initializes a new JSON file configuration provider.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the JSON file
    /// * `optional` - Indicates whether the JSON file must exist
    pub fn new(path: PathBuf, optional: bool) -> Self {
        Self {
            path,
            optional,
            data: HashMap::with_capacity(0),
        }
    }
}

impl ConfigurationProvider for JsonConfigurationProvider {
    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(&key.to_uppercase()).map(|t| t.1.as_str())
    }

    fn load(&mut self) {
        if !self.path.is_file() {
            if self.optional {
                if !self.data.is_empty() {
                    self.data = HashMap::with_capacity(0);
                }

                return;
            } else {
                panic!(
                    "The configuration file '{}' was not found and is not optional.",
                    self.path.display()
                );
            }
        }

        // REF: https://docs.serde.rs/serde_json/de/fn.from_reader.html
        let content = fs::read(&self.path).unwrap();
        let json: Value = serde_json::from_slice(&content).unwrap();

        if let Some(root) = json.as_object() {
            let visitor = JsonVisitor::default();
            self.data = visitor.visit(root);
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
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        accumulate_child_keys(&self.data, earlier_keys, parent_path)
    }
}

/// Represents a [configuration source](trait.ConfigurationSource.html) for JSON files.
pub struct JsonConfigurationSource {
    path: PathBuf,
    optional: bool,
}

impl JsonConfigurationSource {
    /// Initializes a new JSON file configuration source.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the JSON file
    /// * `optional` - Indicates whether the JSON file must exist
    pub fn new(path: &Path, optional: bool) -> Self {
        Self { path: path.to_path_buf(), optional }
    }
}

impl ConfigurationSource for JsonConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(JsonConfigurationProvider::new(
            self.path.clone(),
            self.optional,
        ))
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
        /// * `path` - The path of the JSON file
        fn add_json_file(&mut self, path: &Path) -> &mut Self;

        /// Adds an optional JSON file as a configuration source.
        ///
        /// # Arguments
        ///
        /// * `path` - The path of the JSON file
        fn add_optional_json_file(&mut self, path: &Path) -> &mut Self;
    }

    impl JsonConfigurationExtensions for dyn ConfigurationBuilder {
        fn add_json_file(&mut self, path: &Path) -> &mut Self {
            self.add(Box::new(JsonConfigurationSource::new(path, false)));
            self
        }

        fn add_optional_json_file(&mut self, path: &Path) -> &mut Self {
            self.add(Box::new(JsonConfigurationSource::new(path, true)));
            self
        }
    }

    impl<T: ConfigurationBuilder> JsonConfigurationExtensions for T {
        fn add_json_file(&mut self, path: &Path) -> &mut Self {
            self.add(Box::new(JsonConfigurationSource::new(path, false)));
            self
        }

        fn add_optional_json_file(&mut self, path: &Path) -> &mut Self {
            self.add(Box::new(JsonConfigurationSource::new(path, true)));
            self
        }
    }
}
