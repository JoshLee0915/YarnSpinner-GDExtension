use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use godot::builtin::{array, Callable, Variant};
use godot::prelude::VariantArray;
use yarnspinner::core::YarnValue;
use yarnspinner::prelude::{IntoYarnValueFromNonYarnValue, YarnFn};
use crate::yarn_conversion_utils::YarnConversionUtils;

#[derive(Clone)]
pub struct YarnCallable(Arc<Mutex<Callable>>);

impl YarnCallable {
    pub fn from_callable(callable: Callable) -> Self {
        return Self(Arc::new(Mutex::new(callable)));
    }
}

unsafe impl Send for YarnCallable {}
unsafe impl Sync for YarnCallable {}

impl YarnFn<fn(VariantArray) -> YarnCallableVariant> for YarnCallable {
    type Out = YarnCallableVariant;

    fn call(&self, input: Vec<YarnValue>) -> Self::Out {
        let callable = self.0.lock().unwrap();
        let mut args = array![];
        for arg in input {
            args.push(YarnConversionUtils::yarn_value_to_variant(&arg));
        }
        let result = callable.callv(args);
        return YarnCallableVariant(result);
    }

    fn parameter_types(&self) -> Vec<TypeId> {
        let mut ids = vec![];
        let callable = self.0.lock().unwrap();
        for argument in callable.as_inner().get_bound_arguments().iter_shared() {
            ids.push(YarnCallableVariant(argument).type_id());
        }
        return ids;
    }

    fn return_type(&self) -> TypeId {
        return YarnCallableVariant(Variant::nil()).type_id();
    }
}

#[derive(Clone)]
pub struct YarnCallableVariant(Variant);

impl IntoYarnValueFromNonYarnValue for YarnCallableVariant {
    fn into_yarn_value(self) -> YarnValue {
        return YarnConversionUtils::variant_to_yarn_value(&self.0).unwrap();
    }
}