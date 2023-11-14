#![allow(dyn_drop)]

use crate::{
    util::*, ConfigurationBuilder, ConfigurationPath, ConfigurationProvider, ConfigurationSource,
    FileSource, LoadError, LoadResult,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use tokens::{ChangeToken, FileChangeToken, SharedChangeToken, SingleChangeToken};
use xml_rs::attribute::OwnedAttribute;
use xml_rs::name::OwnedName;
use xml_rs::reader::{EventReader, XmlEvent};

trait LocalNameResolver {
    fn local_name_or_error(&self, element: &OwnedName, line: usize) -> Result<String, String>;
}

impl LocalNameResolver for OwnedName {
    fn local_name_or_error(&self, element: &OwnedName, line: usize) -> Result<String, String> {
        if self.namespace.is_none() {
            Ok(self.local_name.clone())
        } else {
            Err(format!(
                "XML namespaces are not supported. ({}, Line: {})",
                &element.local_name, line
            ))
        }
    }
}

struct Attribute(String, String);

struct Element {
    line: usize,
    element_name: String,
    name: Option<String>,
    sibling_name: String,
    children: HashMap<String, Vec<Rc<RefCell<Element>>>>,
    text: Option<String>,
    attributes: Vec<Attribute>,
}

impl Element {
    fn new(
        element_name: OwnedName,
        attributes: Vec<OwnedAttribute>,
        line: usize,
    ) -> Result<Self, String> {
        let name = get_name(&element_name, &attributes, line)?;
        let local_name = element_name.local_name_or_error(&element_name, line)?;
        let sibling_name = name
            .as_ref()
            .and_then(|n| {
                Some(ConfigurationPath::combine(&[
                    &local_name.to_uppercase(),
                    &n.to_uppercase(),
                ]))
            })
            .unwrap_or(local_name.to_uppercase());

        Ok(Self {
            line,
            element_name: local_name,
            name,
            sibling_name,
            children: HashMap::new(),
            text: None,
            attributes: attributes
                .into_iter()
                .map(|a| {
                    Ok(Attribute(
                        a.name.local_name_or_error(&element_name, line)?,
                        a.value,
                    ))
                })
                .collect::<Result<Vec<Attribute>, String>>()?,
        })
    }
}

#[derive(Default)]
struct Prefix {
    text: String,
    lengths: Vec<usize>,
}

impl Prefix {
    fn push<S: AsRef<str>>(&mut self, value: S) {
        if self.text.is_empty() {
            self.text.push_str(&value.as_ref());
            self.lengths.push(value.as_ref().len());
        } else {
            self.text.push_str(ConfigurationPath::key_delimiter());
            self.text.push_str(&value.as_ref());
            self.lengths
                .push(value.as_ref().len() + ConfigurationPath::key_delimiter().len());
        }
    }

    fn pop(&mut self) {
        if let Some(length) = self.lengths.pop() {
            let idx = self.text.len() - length;
            for _ in 0..length {
                let _ = self.text.remove(idx);
            }
        }
    }
}

impl ToString for Prefix {
    fn to_string(&self) -> String {
        self.text.clone()
    }
}

fn get_name(
    element: &OwnedName,
    attributes: &Vec<OwnedAttribute>,
    line: usize,
) -> Result<Option<String>, String> {
    for attribute in attributes {
        let local_name = attribute.name.local_name_or_error(element, line)?;

        match local_name.as_str() {
            "name" | "Name" | "NAME" => {
                return Ok(Some(attribute.value.clone()));
            }
            _ => {}
        }
    }

    Ok(None)
}

fn process_element(
    prefix: &mut Prefix,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) -> Result<(), String> {
    process_attributes(prefix, element, config)?;
    process_element_content(prefix, element, config)?;
    process_children(prefix, element, config)
}

fn process_element_content(
    prefix: &mut Prefix,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) -> Result<(), String> {
    if let Some(ref value) = element.text {
        add_to_config(prefix.to_string(), value.clone(), element, config)
    } else {
        Ok(())
    }
}

fn process_element_child(
    prefix: &mut Prefix,
    child: &Element,
    index: Option<usize>,
    config: &mut HashMap<String, (String, String)>,
) -> Result<(), String> {
    prefix.push(&child.element_name);

    if let Some(ref name) = child.name {
        prefix.push(name);
    }

    if let Some(i) = index {
        prefix.push(i.to_string());
    }

    process_element(prefix, child, config)?;

    if index.is_some() {
        prefix.pop();
    }

    if child.name.is_some() {
        prefix.pop();
    }

    prefix.pop();
    Ok(())
}

fn process_attributes(
    prefix: &mut Prefix,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) -> Result<(), String> {
    for attribute in &element.attributes {
        prefix.push(&attribute.0);
        add_to_config(prefix.to_string(), attribute.1.clone(), element, config)?;
        prefix.pop();
    }

    Ok(())
}

fn process_children(
    prefix: &mut Prefix,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) -> Result<(), String> {
    for children in element.children.values() {
        if children.len() == 1 {
            process_element_child(prefix, &children[0].deref().borrow(), None, config)?;
        } else {
            for (i, child) in children.iter().enumerate() {
                process_element_child(prefix, &child.deref().borrow(), Some(i), config)?;
            }
        }
    }

    Ok(())
}

fn add_to_config(
    key: String,
    value: String,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) -> Result<(), String> {
    if let Some((dup_key, _)) = config.insert(key.to_uppercase(), (key, value)) {
        Err(format!(
            "A duplicate key '{}' was found. ({}, Line: {})",
            &dup_key, &element.element_name, element.line
        ))
    } else {
        Ok(())
    }
}

fn to_config(
    mut root: Option<Rc<RefCell<Element>>>,
) -> Result<HashMap<String, (String, String)>, String> {
    if let Some(cell) = root.take() {
        let element = &cell.deref().borrow();
        let mut data = HashMap::new();
        let mut prefix = Prefix::default();

        if let Some(ref name) = element.name {
            prefix.push(name);
        }

        process_element(&mut prefix, &element, &mut data)?;
        Ok(data)
    } else {
        Ok(HashMap::with_capacity(0))
    }
}

fn visit(file: File) -> Result<HashMap<String, (String, String)>, String> {
    let content = BufReader::new(file);
    let events = EventReader::new(content);
    let mut has_content = false;
    let mut last_name = None;
    let mut line = 0;
    let mut root = None;
    let mut current = Vec::<Rc<RefCell<Element>>>::new();

    for event in events.into_iter() {
        match event {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                line += 1;
                has_content = false;
                last_name = Some(name.clone());
                let element = Element::new(name, attributes, line)?;
                let key = element.sibling_name.clone();
                let child = Rc::new(RefCell::new(element));

                if let Some(parent) = current.last() {
                    parent
                        .borrow_mut()
                        .children
                        .entry(key)
                        .or_insert(Vec::new())
                        .push(child.clone());
                } else {
                    root = Some(child.clone());
                }

                current.push(child);
            }
            Ok(XmlEvent::EndElement { name }) => {
                if has_content {
                    if let Some(ref last) = last_name {
                        if last != &name {
                            line += 1;
                        }
                    }
                }

                if !current.is_empty() {
                    let _ = current.pop();
                }
            }
            Ok(XmlEvent::CData(text)) | Ok(XmlEvent::Characters(text)) => {
                has_content = true;
                if let Some(parent) = current.last() {
                    parent.borrow_mut().text = Some(text);
                }
            }
            _ => {}
        };
    }

    to_config(root)
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

        if let Ok(file) = File::open(&self.file.path) {
            let data = visit(file).map_err(|e| LoadError::File {
                message: e,
                path: self.file.path.clone(),
            })?;
            *self.data.write().unwrap() = data;
        } else {
            *self.data.write().unwrap() = HashMap::with_capacity(0);
        }

        let previous = std::mem::replace(
            &mut *self.token.write().unwrap(),
            SharedChangeToken::default(),
        );

        previous.notify();
        Ok(())
    }

    fn get(&self, key: &str) -> Option<Cow<String>> {
        self.data
            .read()
            .unwrap()
            .get(&key.to_uppercase())
            .map(|t| Cow::Owned(t.1.clone()))
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(self.token.read().unwrap().clone())
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        let data = self.data.read().unwrap();
        accumulate_child_keys(&data, earlier_keys, parent_path)
    }
}

/// Represents a [configuration provider](trait.ConfigurationProvider.html) for XML files.
pub struct XmlConfigurationProvider {
    inner: Arc<InnerProvider>,
    _registration: Option<Box<dyn Drop>>,
}

impl XmlConfigurationProvider {
    /// Initializes a new XML file configuration provider.
    ///
    /// # Arguments
    ///
    /// * `file` - The [XML file](struct.FileSource.html) information
    pub fn new(file: FileSource) -> Self {
        let path = file.path.clone();
        let inner = Arc::new(InnerProvider::new(file));
        let registration: Option<Box<dyn Drop>> = if inner.file.reload_on_change {
            let other = inner.clone();

            Some(Box::new(tokens::on_change(
                move || FileChangeToken::new(path.clone()),
                move || {
                    std::thread::sleep(other.file.reload_delay);
                    other.load(true).ok();
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

impl ConfigurationProvider for XmlConfigurationProvider {
    fn get(&self, key: &str) -> Option<Cow<String>> {
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

/// Represents a [configuration source](trait.ConfigurationSource.html) for XML files.
pub struct XmlConfigurationSource {
    file: FileSource,
}

impl XmlConfigurationSource {
    /// Initializes a new XML file configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The [XML file](struct.FileSource.html) information
    pub fn new(file: FileSource) -> Self {
        Self { file }
    }
}

impl ConfigurationSource for XmlConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(XmlConfigurationProvider::new(self.file.clone()))
    }
}

pub mod ext {

    use super::*;

    /// Defines extension methods for the [ConfigurationBuilder](trait.ConfigurationBuilder.html) trait.
    pub trait XmlConfigurationExtensions {
        /// Adds a XML file as a configuration source.
        ///
        /// # Arguments
        ///
        /// * `file` - The [XML file](struct.FileSource.html) information
        fn add_xml_file<T: Into<FileSource>>(&mut self, file: T) -> &mut Self;
    }

    impl XmlConfigurationExtensions for dyn ConfigurationBuilder {
        fn add_xml_file<T: Into<FileSource>>(&mut self, file: T) -> &mut Self {
            self.add(Box::new(XmlConfigurationSource::new(file.into())));
            self
        }
    }

    impl<T: ConfigurationBuilder> XmlConfigurationExtensions for T {
        fn add_xml_file<F: Into<FileSource>>(&mut self, file: F) -> &mut Self {
            self.add(Box::new(XmlConfigurationSource::new(file.into())));
            self
        }
    }
}
