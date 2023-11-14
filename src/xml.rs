#![allow(dyn_drop)]

use crate::{
    util::*, ConfigurationBuilder, ConfigurationPath, ConfigurationProvider, ConfigurationSource,
    FileSource,
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
    fn local_name_or_panic(&self) -> String;
}

impl LocalNameResolver for OwnedName {
    fn local_name_or_panic(&self) -> String {
        if self.namespace.is_none() {
            self.local_name.clone()
        } else {
            panic!("XML namespaces are not supported.")
        }
    }
}

struct Attribute(String, String);

struct Element {
    element_name: String,
    name: Option<String>,
    sibling_name: String,
    children: HashMap<String, Vec<Rc<RefCell<Element>>>>,
    text: Option<String>,
    attributes: Vec<Attribute>,
}

impl Element {
    fn new(element_name: OwnedName, attributes: Vec<OwnedAttribute>) -> Self {
        let name = get_name(&attributes);
        let element_name = element_name.local_name_or_panic();
        let sibling_name = name
            .as_ref()
            .and_then(|n| {
                Some(ConfigurationPath::combine(&[
                    &element_name.to_uppercase(),
                    &n.to_uppercase(),
                ]))
            })
            .unwrap_or(element_name.to_uppercase());

        Self {
            element_name,
            name,
            sibling_name,
            children: HashMap::new(),
            text: None,
            attributes: attributes
                .into_iter()
                .map(|a| Attribute(a.name.local_name_or_panic(), a.value))
                .collect(),
        }
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

fn get_name(attributes: &Vec<OwnedAttribute>) -> Option<String> {
    for attribute in attributes {
        match attribute.name.local_name_or_panic().as_str() {
            "name" | "Name" | "NAME" => {
                return Some(attribute.value.clone());
            }
            _ => {}
        }
    }

    None
}

fn process_element(
    prefix: &mut Prefix,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) {
    process_attributes(prefix, element, config);
    process_element_content(prefix, element, config);
    process_children(prefix, element, config);
}

fn process_element_content(
    prefix: &mut Prefix,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) {
    if let Some(ref value) = element.text {
        add_to_config(prefix.to_string(), value.clone(), config);
    }
}

fn process_element_child(
    prefix: &mut Prefix,
    child: &Element,
    index: Option<usize>,
    config: &mut HashMap<String, (String, String)>,
) {
    prefix.push(&child.element_name);

    if let Some(ref name) = child.name {
        prefix.push(name);
    }

    if let Some(i) = index {
        prefix.push(i.to_string());
    }

    process_element(prefix, child, config);

    if index.is_some() {
        prefix.pop();
    }

    if child.name.is_some() {
        prefix.pop();
    }

    prefix.pop();
}

fn process_attributes(
    prefix: &mut Prefix,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) {
    for attribute in &element.attributes {
        prefix.push(&attribute.0);
        add_to_config(prefix.to_string(), attribute.1.clone(), config);
        prefix.pop();
    }
}

fn process_children(
    prefix: &mut Prefix,
    element: &Element,
    config: &mut HashMap<String, (String, String)>,
) {
    for children in element.children.values() {
        if children.len() == 1 {
            process_element_child(prefix, &children[0].deref().borrow(), None, config);
        } else {
            for (i, child) in children.iter().enumerate() {
                process_element_child(prefix, &child.deref().borrow(), Some(i), config);
            }
        }
    }
}

fn add_to_config(key: String, value: String, config: &mut HashMap<String, (String, String)>) {
    if let Some((dup_key, _)) = config.insert(key.to_uppercase(), (key, value)) {
        panic!("A duplicate key '{}' was found.", &dup_key);
    }
}

fn to_config(mut root: Option<Rc<RefCell<Element>>>) -> HashMap<String, (String, String)> {
    if let Some(cell) = root.take() {
        let element = &cell.deref().borrow();
        let mut data = HashMap::new();
        let mut prefix = Prefix::default();

        if let Some(ref name) = element.name {
            prefix.push(name);
        }

        process_element(&mut prefix, &element, &mut data);
        data
    } else {
        HashMap::with_capacity(0)
    }
}

fn visit(file: File) -> HashMap<String, (String, String)> {
    let content = BufReader::new(file);
    let events = EventReader::new(content);

    let mut root = None;
    let mut current = Vec::<Rc<RefCell<Element>>>::new();

    for event in events {
        match event {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                let element = Element::new(name, attributes);
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
            Ok(XmlEvent::EndElement { .. }) => {
                if !current.is_empty() {
                    let _ = current.pop();
                }
            }
            Ok(XmlEvent::CData(text)) | Ok(XmlEvent::Characters(text)) => {
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

        if let Ok(file) = File::open(&self.file.path) {
            let data = visit(file);
            *self.data.write().unwrap() = data;
        } else {
            *self.data.write().unwrap() = HashMap::with_capacity(0);
        }

        let previous = std::mem::replace(
            &mut *self.token.write().unwrap(),
            SharedChangeToken::default(),
        );

        previous.notify();
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

impl ConfigurationProvider for XmlConfigurationProvider {
    fn get(&self, key: &str) -> Option<Cow<String>> {
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
