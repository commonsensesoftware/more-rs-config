use crate::{path, properties::Properties, Error, FileSource, Result, Settings};
use serde_json::{map::Map, Value as JsonValue};
use std::{fs, mem::take};
use tokens::{ChangeToken, FileChangeToken, NeverChangeToken};

fn to_pascal_case(text: &str) -> String {
    let mut chars = text.chars();

    if let Some(first) = chars.next() {
        first.to_uppercase().collect::<String>() + chars.as_str()
    } else {
        String::new()
    }
}

struct JsonVisitor<'a> {
    settings: &'a mut Settings,
    paths: Vec<String>,
}

impl<'a> JsonVisitor<'a> {
    #[inline]
    fn new(settings: &'a mut Settings) -> Self {
        Self {
            settings,
            paths: Vec::new(),
        }
    }
}

impl JsonVisitor<'_> {
    #[inline]
    fn visit(mut self, root: &Map<String, JsonValue>) {
        self.visit_element(root)
    }

    fn visit_element(&mut self, element: &Map<String, JsonValue>) {
        if element.is_empty() {
            if let Some(key) = self.paths.last() {
                self.settings.insert(to_pascal_case(key), String::new());
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
        let key = self.paths.last().expect("no paths").to_string();
        self.settings.insert(key, value.to_string());
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

struct Provider(FileSource);

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Json"
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

        // REF: https://docs.serde.rs/serde_json/de/fn.from_reader.html
        let content = fs::read(&self.0.path).map_err(Error::unknown)?;
        let json: JsonValue = serde_json::from_slice(&content).map_err(Error::unknown)?;

        if let Some(root) = json.as_object() {
            Ok(JsonVisitor::new(settings).visit(root))
        } else {
            Err(Error::InvalidFile {
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
                path: self.0.path.clone(),
            })
        }
    }
}

/// Represents a [configuration source](Source) for `*.json` files.
pub struct Source(FileSource);

impl Source {
    /// Initializes a new `*.json` file configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.json` [file source](FileSource) information
    #[inline]
    pub fn new(file: FileSource) -> Self {
        Self(file)
    }
}

impl crate::Source for Source {
    #[inline]
    fn build(&mut self, _properties: &mut Properties) -> Box<dyn crate::Provider> {
        Box::new(Provider(take(&mut self.0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_pascal_case_should_normalize_argument_name() {
        // arrange
        let argument = "noBuild";

        // act
        let pascal_case = to_pascal_case(argument);

        // assert
        assert_eq!(pascal_case, "NoBuild");
    }
}
