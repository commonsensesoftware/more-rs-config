use crate::{pascal_case, path, Error, FileSource, Result, Settings};
use std::fs;
use tokens::{ChangeToken, FileChangeToken, NeverChangeToken};
use yaml_rust2::{yaml::Hash, Yaml, YamlLoader};

struct YamlVisitor<'a> {
    settings: &'a mut Settings,
    paths: Vec<String>,
}

impl<'a> YamlVisitor<'a> {
    #[inline]
    fn new(settings: &'a mut Settings) -> Self {
        Self {
            settings,
            paths: Vec::new(),
        }
    }
}

impl YamlVisitor<'_> {
    #[inline]
    fn visit(mut self, root: &Hash) {
        self.visit_hash(root)
    }

    fn visit_hash(&mut self, hash: &Hash) {
        if hash.is_empty() {
            if let Some(key) = self.paths.last() {
                self.settings.insert(pascal_case(key), String::new());
            }
        } else {
            for (name, value) in hash {
                let key = match name {
                    Yaml::String(s) => pascal_case(s),
                    Yaml::Integer(i) => i.to_string(),
                    Yaml::Real(s) => s.clone(),
                    Yaml::Boolean(b) => b.to_string(),
                    _ => String::new(),
                };
                self.enter_context(key);
                self.visit_value(value);
                self.exit_context();
            }
        }
    }

    fn visit_value(&mut self, value: &Yaml) {
        match value {
            Yaml::Hash(ref hash) => self.visit_hash(hash),
            Yaml::Array(array) => {
                for (index, element) in array.iter().enumerate() {
                    self.enter_context(index.to_string());
                    self.visit_value(element);
                    self.exit_context();
                }
            }
            Yaml::String(value) => self.add_value(value),
            Yaml::Integer(value) => self.add_value(value),
            Yaml::Real(value) => self.add_value(value),
            Yaml::Boolean(value) => self.add_value(value),
            Yaml::Null => self.add_value(String::new()),
            Yaml::Alias(_) | Yaml::BadValue => self.add_value(String::new()),
        }
    }

    fn add_value<T: ToString>(&mut self, value: T) {
        let key = self.paths.last().expect("no paths");
        self.settings.insert(pascal_case(key), value.to_string());
    }

    fn enter_context(&mut self, context: String) {
        if self.paths.is_empty() {
            self.paths.push(context);
        } else {
            self.paths
                .push(path::combine(&[&self.paths[self.paths.len() - 1], &context]));
        }
    }

    #[inline]
    fn exit_context(&mut self) {
        self.paths.pop();
    }
}

/// Represents a [configuration provider](crate::Provider) for `*.yaml` and `*.yml` files.
pub struct Provider(FileSource);

impl Provider {
    /// Initializes a new `*.yaml` file configuration provider.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.yaml` [file source](FileSource) information
    #[inline]
    pub fn new(file: FileSource) -> Self {
        Self(file)
    }
}

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Yaml"
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        if self.0.reload_on_change {
            Box::new(FileChangeToken::new(self.0.path.clone()))
        } else {
            Box::new(NeverChangeToken)
        }
    }

    fn load(&self, settings: &mut Settings) -> Result {
        if !self.0.path.is_file() {
            if self.0.optional {
                return Ok(());
            } else {
                return Err(Error::MissingFile(self.0.path.clone()));
            }
        }

        let content = fs::read_to_string(&self.0.path).map_err(Error::unknown)?;
        let docs = YamlLoader::load_from_str(&content).map_err(|e| Error::InvalidFile {
            message: e.to_string(),
            path: self.0.path.clone(),
        })?;

        if docs.is_empty() {
            return Ok(());
        }

        let doc = &docs[0];

        match doc {
            Yaml::Hash(ref hash) => {
                YamlVisitor::new(settings).visit(hash);
                Ok(())
            }
            _ => Err(Error::InvalidFile {
                message: format!(
                    "Top-level YAML element must be a mapping, but '{}' was found.",
                    match doc {
                        Yaml::Array(_) => "array",
                        Yaml::String(_) => "string",
                        Yaml::Integer(_) => "integer",
                        Yaml::Real(_) => "float",
                        Yaml::Boolean(_) => "Boolean",
                        Yaml::Null => "null",
                        _ => "unknown",
                    }
                ),
                path: self.0.path.clone(),
            }),
        }
    }
}
