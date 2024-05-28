use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use godot::global::push_warning;

use godot::prelude::*;
use yarnspinner::core::YarnValue;
use yarnspinner::prelude::VariableStorage;
use yarnspinner::runtime::VariableStorageError;
use yarnspinner::runtime::VariableStorageError::{InternalError, InvalidVariableName, VariableNotFound};
use crate::yarn_conversion_utils::YarnConversionUtils;

#[derive(GodotConvert, Var, Export, Default, Debug)]
#[godot(via = GString)]
pub enum YarnVariableSetResult {
    Ok,
    InvalidVariableName,
    #[default]
    Unknown,
}

#[derive(GodotClass, Debug)]
#[class(init, base=Node)]
pub struct YarnVariableStorage {
    base: Base<Node>,
    #[export]
    store: Dictionary,
}

// TODO: Look into using virtual when it is released with the godot 4.3 version
// https://godot-rust.github.io/book/register/virtual-functions.html
#[godot_api]
impl YarnVariableStorage {
    #[signal]
    fn variable_changed(variable_name: GString, new_value: Variant) {}
    #[signal]
    fn store_cleared() {}

    #[func]
    pub fn get_variables(&self) -> Dictionary {
        return self.store.clone();
    }
    #[func]
    pub fn get_variable(&self, variable_name: GString) -> Variant {
        return self.store.get_or_nil(variable_name);
    }
    #[func]
    pub fn set_variable(&mut self, variable_name: GString, value: Variant) -> YarnVariableSetResult {
        return match Self::validate_name(variable_name.to_string()) {
            Ok(_) => {
                self.store.set(variable_name.to_variant(), value.clone());
                self.base_mut().emit_signal(StringName::from("variable_changed"), &[variable_name.to_variant(), value.clone()]);
                YarnVariableSetResult::Ok
            }
            Err(_) => {
                push_warning(&[format!("Variable name {} is an invalid format. Yarn variables must start with a '$'", variable_name.clone()).to_variant()]);
                YarnVariableSetResult::InvalidVariableName
            },
        }
    }
    #[func]
    pub fn set_variables(&mut self, values: Dictionary) -> Dictionary {
        let mut results = dict! {};

        for (key, value) in values.iter_shared() {
            let result = self.set_variable(key.stringify(), value);
            results.set(key, result);
        }

        return results;
    }
    #[func]
    pub fn clear(&mut self) {
        self.store.clear();
        self.base_mut().emit_signal(StringName::from("store_cleared"), &[]);
    }
    #[func]
    pub fn contains(&self, variable_name: GString) -> bool {
        return self.store.contains_key(variable_name);
    }
}

// Taken from the MemoryVariableStorage check
impl YarnVariableStorage {
    fn validate_name(name: impl AsRef<str>) -> Result<(), VariableStorageError> {
        let name = name.as_ref();
        if name.starts_with('$') {
            Ok(())
        } else {
            Err(VariableStorageError::InvalidVariableName {
                name: name.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariableStorageWrapper {
    store: Arc<Mutex<Gd<YarnVariableStorage>>>
}

impl VariableStorageWrapper {
    pub fn wrap(store: &Gd<YarnVariableStorage>) -> Box<dyn VariableStorage> {
        return Box::new(Self{ store: Arc::new(Mutex::new(store.clone())) });
    }
}

unsafe impl Send for VariableStorageWrapper {}
unsafe impl Sync for VariableStorageWrapper {}
impl VariableStorage for VariableStorageWrapper {
    fn clone_shallow(&self) -> Box<dyn VariableStorage> {
        return Box::new(self.clone());
    }

    fn set(&mut self, name: String, value: YarnValue) -> Result<(), VariableStorageError> {
        return match self.store.lock().unwrap().bind_mut().set_variable(name.to_godot(), YarnConversionUtils::yarn_value_to_variant(&value)) {
            YarnVariableSetResult::Ok => Ok(()),
            YarnVariableSetResult::InvalidVariableName => Err(InvalidVariableName{name: name.clone()}),
            YarnVariableSetResult::Unknown => Err(InternalError{error: format!("Failed to set {} to {}", name.clone(), value.clone()).into()}),
        }
    }

    fn get(&self, name: &str) -> Result<YarnValue, VariableStorageError> {
        let value = self.store.lock().unwrap().bind().get_variable(name.to_godot());
        if value.is_nil() {
            return Err(VariableNotFound {name: name.to_string()})
        }

        return match YarnConversionUtils::variant_to_yarn_value(&value) {
            Ok(yarn_value) => Ok(yarn_value),
            Err(err) => Err(err),
        }
    }

    fn contains(&self, name: &str) -> bool {
        return self.store.lock().unwrap().bind().contains(name.to_godot());
    }

    fn extend(&mut self, values: HashMap<String, YarnValue>) -> Result<(), VariableStorageError> {
        for (key, value) in values {
            match self.set(key, value) {
                Ok(_) => {},
                Err(err) => return Err(err),
            }
        }
        return Ok(());
    }

    fn variables(&self) -> HashMap<String, YarnValue> {
        let mut hash_map = HashMap::new();
        for (key, value) in self.store.lock().unwrap().bind().get_variables().iter_shared() {
            hash_map.insert(key.to_string(), YarnConversionUtils::variant_to_yarn_value(&value).unwrap());
        }
        return hash_map;
    }

    fn clear(&mut self) {
        self.store.lock().unwrap().bind_mut().clear();
    }

    fn as_any(&self) -> &dyn Any {
        return self;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        return self;
    }
}