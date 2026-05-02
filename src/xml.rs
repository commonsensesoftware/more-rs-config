use crate::{pascal_case, path, Error, FileSource, Settings};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    fs::File,
    io::BufReader,
    ops::Deref,
    rc::Rc,
};
use tokens::{ChangeToken, FileChangeToken, NeverChangeToken};
use xml_rs::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent::*},
};

trait LocalNameResolver {
    fn local_name_or_error(&self, element: &OwnedName, line: usize) -> Result<String, String>;
}

impl LocalNameResolver for OwnedName {
    fn local_name_or_error(&self, element: &OwnedName, line: usize) -> Result<String, String> {
        if self.namespace.is_none() {
            Ok(self.local_name.clone())
        } else {
            Err(format!(
                "XML namespaces are not supported. ({name}, Line: {line})",
                name = &element.local_name
            ))
        }
    }
}

trait VecExt<TKey: PartialEq, TValue> {
    fn get_or_add(&mut self, key: TKey) -> &mut TValue;
}

impl VecExt<String, Vec<Rc<RefCell<Element>>>> for Vec<(String, Vec<Rc<RefCell<Element>>>)> {
    fn get_or_add(&mut self, key: String) -> &mut Vec<Rc<RefCell<Element>>> {
        let index = self.iter_mut().position(|i| i.0 == key).unwrap_or(self.len());

        if index == self.len() {
            self.push((key, Vec::new()));
        }

        &mut self[index].1
    }
}

struct Attribute(String, String);

struct Element {
    line: usize,
    element_name: String,
    name: Option<String>,
    sibling_name: String,
    children: Vec<(String, Vec<Rc<RefCell<Element>>>)>,
    text: Option<String>,
    attributes: Vec<Attribute>,
}

impl Element {
    fn new(element_name: OwnedName, attributes: Vec<OwnedAttribute>, line: usize) -> Result<Self, String> {
        let name = get_name(&element_name, &attributes, line)?;
        let local_name = element_name.local_name_or_error(&element_name, line)?;
        let sibling_name = name
            .as_ref()
            .map(|n| path::combine(&[&local_name.to_uppercase(), &n.to_uppercase()]))
            .unwrap_or(local_name.to_uppercase());

        Ok(Self {
            line,
            element_name: local_name,
            name,
            sibling_name,
            children: Default::default(),
            text: None,
            attributes: attributes
                .into_iter()
                .map(|a| Ok(Attribute(a.name.local_name_or_error(&element_name, line)?, a.value)))
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
            self.text.push_str(value.as_ref());
            self.lengths.push(value.as_ref().len());
        } else {
            self.text.push(path::delimiter());
            self.text.push_str(value.as_ref());
            self.lengths.push(value.as_ref().len() + 1);
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

impl Display for Prefix {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

fn get_name(element: &OwnedName, attributes: &Vec<OwnedAttribute>, line: usize) -> Result<Option<String>, String> {
    for attribute in attributes {
        let local_name = attribute.name.local_name_or_error(element, line)?;

        if local_name.eq_ignore_ascii_case("name") {
            return Ok(Some(attribute.value.clone()));
        }
    }

    Ok(None)
}

fn visit_element(prefix: &mut Prefix, element: &Element, settings: &mut Settings) -> Result<(), String> {
    visit_attributes(prefix, element, settings)?;
    visit_element_content(prefix, element, settings)?;
    visit_children(prefix, element, settings)
}

fn visit_element_content(prefix: &mut Prefix, element: &Element, settings: &mut Settings) -> Result<(), String> {
    if let Some(ref value) = element.text {
        add_setting(prefix.to_string(), value.clone(), element, settings)
    } else {
        Ok(())
    }
}

fn visit_element_child(
    prefix: &mut Prefix,
    child: &Element,
    index: Option<usize>,
    settings: &mut Settings,
) -> Result<(), String> {
    prefix.push(&child.element_name);

    if let Some(ref name) = child.name {
        prefix.push(name);
    }

    if let Some(i) = index {
        prefix.push(i.to_string());
    }

    visit_element(prefix, child, settings)?;

    if index.is_some() {
        prefix.pop();
    }

    if child.name.is_some() {
        prefix.pop();
    }

    prefix.pop();
    Ok(())
}

fn visit_attributes(prefix: &mut Prefix, element: &Element, settings: &mut Settings) -> Result<(), String> {
    for attribute in &element.attributes {
        prefix.push(&attribute.0);
        add_setting(prefix.to_string(), attribute.1.clone(), element, settings)?;
        prefix.pop();
    }

    Ok(())
}

fn visit_children(prefix: &mut Prefix, element: &Element, settings: &mut Settings) -> Result<(), String> {
    for children in element.children.iter().map(|i| &i.1) {
        if children.len() == 1 {
            visit_element_child(prefix, &children[0].deref().borrow(), None, settings)?;
        } else {
            for (i, child) in children.iter().enumerate() {
                visit_element_child(prefix, &child.deref().borrow(), Some(i), settings)?;
            }
        }
    }

    Ok(())
}

fn add_setting(key: String, value: String, element: &Element, settings: &mut Settings) -> Result<(), String> {
    if settings.insert(pascal_case(&key), value).is_some() {
        Err(format!(
            "A duplicate key '{key}' was found. ({name}, Line: {line})",
            name = &element.element_name,
            line = element.line,
        ))
    } else {
        Ok(())
    }
}

fn visit(file: File, settings: &mut Settings) -> Result<(), String> {
    let content = BufReader::new(file);
    let events = EventReader::new(content);
    let mut has_content = false;
    let mut last_name = None;
    let mut line = 0;
    let mut root = None;
    let mut current = Vec::<(OwnedName, Rc<RefCell<Element>>)>::new();

    for event in events.into_iter() {
        match event {
            Ok(StartElement { name, attributes, .. }) => {
                line += 1;
                has_content = false;
                last_name = Some(name.clone());
                let element = Element::new(name.clone(), attributes, line)?;
                let key = element.sibling_name.clone();
                let child = Rc::new(RefCell::new(element));

                if let Some(parent) = current.last() {
                    parent.1.borrow_mut().children.get_or_add(key).push(child.clone());
                } else {
                    root = Some(child.clone());
                }

                current.push((name, child));
            }
            Ok(EndElement { name }) => {
                if has_content {
                    if let Some(ref last) = last_name {
                        if last != &name {
                            line += 1;
                        }
                    }
                }

                if let Some((current_name, _)) = current.pop() {
                    last_name = Some(current_name);
                }
            }
            Ok(CData(text)) | Ok(Characters(text)) => {
                has_content = true;
                if let Some(parent) = current.last() {
                    parent.1.borrow_mut().text = Some(text);
                }
            }
            _ => {}
        };
    }

    if let Some(cell) = root.take() {
        let element = &cell.deref().borrow();
        let mut prefix = Prefix::default();

        if let Some(ref name) = element.name {
            prefix.push(name);
        }

        visit_element(&mut prefix, element, settings)?;
    }

    Ok(())
}

/// Represents a [configuration provider](Provider) for `*.xml` files.
pub struct Provider(FileSource);

impl Provider {
    /// Initializes a new `*.xml` file configuration provider.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.xml` [file source](FileSource) information
    #[inline]
    pub fn new(file: FileSource) -> Self {
        Self(file)
    }
}

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Xml"
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        if self.0.reload_on_change {
            Box::new(FileChangeToken::new(self.0.path.clone()))
        } else {
            Box::new(NeverChangeToken)
        }
    }

    fn load(&self, settings: &mut Settings) -> crate::Result {
        if !self.0.path.is_file() {
            if self.0.optional {
                return Ok(());
            } else {
                return Err(Error::MissingFile(self.0.path.clone()));
            }
        }

        let file = File::open(&self.0.path).map_err(Error::unknown)?;

        visit(file, settings).map_err(|e| Error::InvalidFile {
            message: e,
            path: self.0.path.clone(),
        })?;

        Ok(())
    }
}
