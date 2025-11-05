use godot::global::push_warning;
use std::any::Any;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use bevy_platform::collections::HashMap;
use crate::yarn_conversion_utils::YarnConversionUtils;
use godot::prelude::*;
use yarnspinner::core::YarnValue;
use yarnspinner::prelude::VariableStorage;
use yarnspinner::runtime::VariableStorageError;
use yarnspinner::runtime::VariableStorageError::{InternalError, InvalidVariableName, VariableNotFound};

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
    default_store: Dictionary,
}

// TODO: Look into using virtual when it is released with the godot 4.3 version
// https://godot-rust.github.io/book/register/virtual-functions.html
#[godot_api]
impl YarnVariableStorage {
    #[signal]
    fn variable_changed(variable_name: GString, new_value: Variant);
    #[signal]
    fn store_cleared();

    #[func(gd_self)]
    pub fn contains(instance :Gd<Self>, variable_name: GString) -> bool {
        return instance.bind().contains_impl(variable_name);
    }

    #[func(virtual)]
    fn contains_impl(&self, variable_name: GString) -> bool {
        return self.default_store.contains_key(variable_name);
    }

    #[func(gd_self)]
    pub fn get_variables(instance :Gd<Self>) -> Dictionary {
        return instance.bind().get_variables_impl()
    }

    #[func(virtual)]
    fn get_variables_impl(&self) -> Dictionary { return self.default_store.clone(); }

    #[func(gd_self)]
    pub fn get_variable(instance :Gd<Self>, variable_name: GString) -> Variant {
        return instance.bind().get_variable_impl(variable_name);
    }

    #[func(virtual)]
    fn get_variable_impl(&self, variable_name: GString) -> Variant  {
        return self.default_store.get_or_nil(variable_name);
    }

    #[func(gd_self)]
    pub fn set_variable(mut instance :Gd<Self>, variable_name: GString, value: Variant) -> YarnVariableSetResult {
        return match Self::validate_name(variable_name.to_string()) {
            Ok(_) => {
                instance.bind_mut().set_variable_impl(variable_name.clone(), value.clone());
                instance.signals().variable_changed().emit(&variable_name, &value);
                YarnVariableSetResult::Ok
            }
            Err(_) => {
                push_warning(&[format!("Variable name {} is an invalid format. Yarn variables must start with a '$'", variable_name.clone()).to_variant()]);
                YarnVariableSetResult::InvalidVariableName
            },
        }
    }

    #[func(virtual)]
    fn set_variable_impl(&mut self, variable_name: GString, value: Variant) {
        self.default_store.set(variable_name.to_variant(), value.clone());
    }

    #[func(gd_self)]
    pub fn set_variables(instance :Gd<Self>, values: Dictionary) -> Dictionary {
        return Self::set_variables_impl(instance, values);
    }

    #[func(virtual, gd_self)]
    fn set_variables_impl(instance :Gd<Self>, values: Dictionary) -> Dictionary {
        let mut results = vdict! {};

        for (key, value) in values.iter_shared() {
            let result = Self::set_variable(instance.clone(), key.stringify(), value);
            results.set(key, result);
        }

        return results;
    }

    #[func(gd_self)]
    pub fn clear(mut instance :Gd<Self>) {
        instance.bind_mut().clear_impl();
        instance.signals().store_cleared().emit();
    }

    #[func(virtual)]
    fn clear_impl(&mut self) {
        self.default_store.clear();
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
        return match YarnVariableStorage::set_variable((*self.store.lock().unwrap()).clone(), name.to_godot(), YarnConversionUtils::yarn_value_to_variant(&value)) {
            YarnVariableSetResult::Ok => Ok(()),
            YarnVariableSetResult::InvalidVariableName => Err(InvalidVariableName{name: name.clone()}),
            YarnVariableSetResult::Unknown => Err(InternalError{error: format!("Failed to set {} to {}", name.clone(), value.clone()).into()}),
        }
    }

    fn get(&self, name: &str) -> Result<YarnValue, VariableStorageError> {
        let value = YarnVariableStorage::get_variable((*self.store.lock().unwrap()).clone(), name.to_godot());
        if value.is_nil() {
            return Err(VariableNotFound {name: name.to_string()})
        }

        return match YarnConversionUtils::variant_to_yarn_value(&value) {
            Ok(yarn_value) => Ok(yarn_value),
            Err(err) => Err(err),
        }
    }

    fn contains(&self, name: &str) -> bool {
        return YarnVariableStorage::contains((*self.store.lock().unwrap()).clone(), name.to_godot());
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
        for (key, value) in YarnVariableStorage::get_variables((*self.store.lock().unwrap()).clone()).iter_shared() {
            hash_map.insert(key.to_string(), YarnConversionUtils::variant_to_yarn_value(&value).unwrap());
        }
        return hash_map;
    }

    fn clear(&mut self) {
        YarnVariableStorage::clear((*self.store.lock().unwrap()).clone());
    }

    fn as_any(&self) -> &dyn Any {
        return self;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        return self;
    }
}