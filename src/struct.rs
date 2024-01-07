use crate::{
    ConfigurationBuilder, ConfigurationPath, ConfigurationProvider, ConfigurationSource,
    LoadError, LoadResult, Value, accumulate_child_keys, to_pascal_case
};

use serde_intermediate::value::intermediate::Intermediate;
use std::collections::HashMap;
use std::sync::RwLock;
use serde::Serialize;
use std::sync::Arc;

#[derive(Default)]
struct StructVisitor {
    data: HashMap<String, (String, Value)>,
    paths: Vec<String>,
}

impl StructVisitor {
    fn visit(mut self, root: &Intermediate) -> HashMap<String, (String, Value)> {
        self.visit_element(root);
        self.data.shrink_to_fit();
        self.data
    }

    fn visit_element(&mut self, element: &Intermediate) {
        match element {
            Intermediate::Unit |
            Intermediate::UnitStruct |
            Intermediate::UnitVariant(_) => {
                if let Some(key) = self.paths.last() {
                    self.data
                        .insert(key.to_uppercase(), (to_pascal_case(key), String::new().into()));
                }
                return
            },
            _ => {}
        };

        match element {
            Intermediate::Seq(vector) |
            Intermediate::Tuple(vector) |
            Intermediate::TupleVariant(_, vector) |
            Intermediate::TupleStruct(vector) => {
                if vector.len() > 0
                {
                    for (index, element) in vector.iter().enumerate() {
                        self.enter_context(index.to_string());
                        self.visit_element(element);
                        self.exit_context();
                    }
                } else {
                    if let Some(key) = self.paths.last() {
                        self.data
                            .insert(key.to_uppercase(), (to_pascal_case(key), String::new().into()));
                    }
                }
                return
            },
            _ => {}
        };

        match element {
            Intermediate::Map(map) => {
                if map.len() > 0
                {
                    for (name, element) in map {
                        self.enter_context(name.to_string());
                        self.visit_element(element);
                        self.exit_context();
                    }
                } else {
                    if let Some(key) = self.paths.last() {
                        self.data
                            .insert(key.to_uppercase(), (to_pascal_case(key), String::new().into()));
                    }
                }
                return
            },
            Intermediate::Struct(map) |
            Intermediate::StructVariant(_, map) => {
                if map.len() > 0
                {
                    for (name, element) in map {
                        self.enter_context(name.to_string());
                        self.visit_element(element);
                        self.exit_context();
                    }
                } else {
                    if let Some(key) = self.paths.last() {
                        self.data
                            .insert(key.to_uppercase(), (to_pascal_case(key), String::new().into()));
                    }
                }
                return
            },
            _ => {}
        };

            //Intermediate::Bytes(v) => { /* vec<u8> */ },
        match element {
            Intermediate::Bool(v) => { self.add_value(v) },
            Intermediate::String(v) => { self.add_value(v) },
            Intermediate::I8(v) => { self.add_value(v) },
            Intermediate::I16(v) => { self.add_value(v) },
            Intermediate::I32(v) => { self.add_value(v) },
            Intermediate::I64(v) => { self.add_value(v) },
            Intermediate::I128(v) => { self.add_value(v) },
            Intermediate::U8(v) => { self.add_value(v) },
            Intermediate::U16(v) => { self.add_value(v) },
            Intermediate::U32(v) => { self.add_value(v) },
            Intermediate::U64(v) => { self.add_value(v) },
            Intermediate::U128(v) => { self.add_value(v) },
            Intermediate::F32(v) => { self.add_value(v) },
            Intermediate::F64(v) => { self.add_value(v) },
            Intermediate::Char(v) => { self.add_value(v) },
            Intermediate::Option(v) => {
                match v {
                    Some(v) => self.add_value(v),
                    None => {},
                };
            },
            _ => { std::unreachable!(); }
        };
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

struct InnerProvider<T: Serialize> {
    r#struct: Box<T>,
    data: RwLock<HashMap<String, (String, Value)>>,
}

impl<T: Serialize> InnerProvider<T> {
    fn new(r#struct: T) -> Self {
        Self {
            r#struct: Box::new(r#struct),
            data: RwLock::new(HashMap::with_capacity(0)),
        }
    }

    fn load(&self) -> LoadResult {
        let visitor = StructVisitor::default();
        let data = visitor.visit(match &serde_intermediate::to_intermediate(&self.r#struct) {
            Ok(v) => v,
            Err(_) => { Err(LoadError::Generic(String::from("Could not serialize data")))? },
        });
        *match self.data.write() {
            Ok(ptr) => ptr,
            Err(_) => { Err(LoadError::Generic(String::from("Could not get lock to write data")))? },
        } = data;
        Ok(())
    }

    fn get(&self, key: &str) -> Option<Value> {
        self.data
            .read()
            .unwrap()
            .get(&key.to_uppercase())
            .map(|t| t.1.clone())
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        let data = self.data.read().unwrap();
        accumulate_child_keys(&data, earlier_keys, parent_path)
    }
}

pub struct StructConfigurationProvider<T: Serialize> {
    inner: Arc<InnerProvider<T>>,
}

impl<T: Serialize> StructConfigurationProvider<T> {
    pub fn new(obj: T) -> Self {
        let obj = Self {
            inner: Arc::new(InnerProvider::new(obj))
        };
        let _ = obj.inner.load();
        obj
    }
}

impl<T: Serialize> ConfigurationProvider for StructConfigurationProvider<T> {
    fn get(&self, key: &str) -> Option<Value> {
        self.inner.get(key)
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        self.inner.child_keys(earlier_keys, parent_path)
    }
}

pub struct StructConfigurationSource<T: Serialize> {
    obj: T
}

impl<T: Serialize> StructConfigurationSource<T> {
    pub fn new(obj: T) -> Self {
        Self {
            obj
        }
    }
}

impl<T: Serialize + Clone + 'static> ConfigurationSource for StructConfigurationSource<T> {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(StructConfigurationProvider::new(self.obj.clone()))
    }
}

pub mod ext {

    use super::*;

    pub trait StructConfigurationExtensions {
        fn add_struct<T: Serialize + Clone + 'static>(&mut self, r#struct: T) -> &mut Self;
    }

    impl StructConfigurationExtensions<> for dyn ConfigurationBuilder {
        fn add_struct<T: Serialize + Clone + 'static>(&mut self, r#struct: T) -> &mut Self {
            self.add(Box::new(StructConfigurationSource::new(r#struct)));
            self
        }
    }

    impl<T: ConfigurationBuilder> StructConfigurationExtensions for T {
        fn add_struct<F: Serialize + Clone + 'static>(&mut self, r#struct: F) -> &mut Self {
            self.add(Box::new(StructConfigurationSource::new(r#struct)));
            self
        }
    }
}

