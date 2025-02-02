use crate::{
    util::*, ConfigurationBuilder, ConfigurationPath, ConfigurationProvider, ConfigurationSource, FileSource,
    LoadError, LoadResult, Value,
};
use serde_yaml::{Mapping, Value as YamlValue};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, RwLock};
use tokens::{ChangeToken, FileChangeToken, SharedChangeToken, SingleChangeToken, Subscription};

/// Visits and processes YAML data to extract configuration key-value pairs.
#[derive(Default)]
struct YamlVisitor {
    data: HashMap<String, (String, Value)>,
    paths: Vec<String>,
}

impl YamlVisitor {
    fn visit(mut self, root: &Mapping) -> HashMap<String, (String, Value)> {
        self.visit_element(root);
        self.data.shrink_to_fit();
        self.data
    }

    fn visit_element(&mut self, element: &Mapping) {
        if element.is_empty() {
            if let Some(key) = self.paths.last() {
                self.data
                    .insert(key.to_uppercase(), (key.to_string(), String::new().into()));
            }
        } else {
            for (name, value) in element {
                if let YamlValue::String(key) = name {
                    self.enter_context(key.to_string());
                    self.visit_value(value);
                    self.exit_context();
                }
            }
        }
    }

    fn visit_value(&mut self, value: &YamlValue) {
        match value {
            YamlValue::Mapping(element) => self.visit_element(element),
            YamlValue::Sequence(array) => {
                for (index, element) in array.iter().enumerate() {
                    self.enter_context(index.to_string());
                    self.visit_value(element);
                    self.exit_context();
                }
            }
            YamlValue::Bool(value) => self.add_value(value),
            YamlValue::Null => self.add_value(String::new()),
            YamlValue::Number(value) => self.add_value(value),
            YamlValue::String(value) => self.add_value(value),
            YamlValue::Tagged(tagged) => self.visit_value(&tagged.value),
        }
    }

    fn add_value<T: ToString>(&mut self, value: T) {
        let key = self.paths.last().unwrap().to_string();
        self.data.insert(key.to_uppercase(), (key, value.to_string().into()));
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

        let content = fs::read_to_string(&self.file.path).map_err(|e| LoadError::File {
            message: format!("Failed to read file: {}", e),
            path: self.file.path.clone(),
        })?;
        let yaml: YamlValue = serde_yaml::from_str(&content).map_err(|e| LoadError::File {
            message: format!("Failed to parse YAML: {}", e),
            path: self.file.path.clone(),
        })?;

        if let YamlValue::Mapping(root) = yaml {
            let visitor = YamlVisitor::default();
            let data = visitor.visit(&root);
            *self.data.write().unwrap() = data;
        } else if reload {
            *self.data.write().unwrap() = HashMap::with_capacity(0);
        } else {
            return Err(LoadError::File {
                message: format!(
                    "Top-level YAML element must be a mapping. Instead, '{}' was found.",
                    match yaml {
                        YamlValue::Sequence(_) => "sequence",
                        YamlValue::Bool(_) => "Boolean",
                        YamlValue::Null => "null",
                        YamlValue::Number(_) => "number",
                        YamlValue::String(_) => "string",
                        _ => unreachable!(),
                    }
                ),
                path: self.file.path.clone(),
            });
        }

        let previous = std::mem::take(&mut *self.token.write().unwrap());

        previous.notify();
        Ok(())
    }

    fn get(&self, key: &str) -> Option<Value> {
        self.data.read().unwrap().get(&key.to_uppercase()).map(|t| t.1.clone())
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(self.token.read().unwrap().clone())
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        let data = self.data.read().unwrap();
        accumulate_child_keys(&data, earlier_keys, parent_path)
    }
}

/// Represents a [`ConfigurationProvider`](crate::ConfigurationProvider) for `*.yaml` files.
pub struct YamlConfigurationProvider {
    inner: Arc<InnerProvider>,
    _subscription: Option<Box<dyn Subscription>>,
}

/// Initializes a new `*.yaml` file configuration provider.
///
/// # Arguments
///
/// * `file` - The `*.yaml` [`FileSource`](crate::FileSource) information
impl YamlConfigurationProvider {
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
                Some(inner.clone()),
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

impl ConfigurationProvider for YamlConfigurationProvider {
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

/// Represents a [`ConfigurationSource`](crate::ConfigurationSource) for `*.yaml` files.
pub struct YamlConfigurationSource {
    file: FileSource,
}

impl YamlConfigurationSource {
    /// Initializes a new `*.yaml` file configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.yaml` [`FileSource`](crate::FileSource) information
    pub fn new(file: FileSource) -> Self {
        Self { file }
    }
}

impl ConfigurationSource for YamlConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(YamlConfigurationProvider::new(self.file.clone()))
    }
}

pub mod ext {
    use super::*;

    pub trait YamlConfigurationExtensions {
        /// Adds a `*.yaml` file as a configuration source.
        ///
        /// # Arguments
        ///
        /// * `file` - The `*.yaml` [`FileSource`](crate::FileSource) information
        fn add_yaml_file<T: Into<FileSource>>(&mut self, file: T) -> &mut Self;
    }

    impl YamlConfigurationExtensions for dyn ConfigurationBuilder + '_ {
        fn add_yaml_file<T: Into<FileSource>>(&mut self, file: T) -> &mut Self {
            self.add(Box::new(YamlConfigurationSource::new(file.into())));
            self
        }
    }

    impl<T: ConfigurationBuilder> YamlConfigurationExtensions for T {
        fn add_yaml_file<F: Into<FileSource>>(&mut self, file: F) -> &mut Self {
            self.add(Box::new(YamlConfigurationSource::new(file.into())));
            self
        }
    }
}
