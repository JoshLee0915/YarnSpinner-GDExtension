use crate::yarn_conversion_utils::YarnConversionUtils;
use godot::builtin::{array, Callable, Variant};
use godot::prelude::{VariantArray, VariantType};
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use yarnspinner::core::YarnValue;
use yarnspinner::prelude::{IntoYarnValueFromNonYarnValue, YarnFn};

#[derive(Clone)]
pub struct YarnCallable
{
    pub callable: Arc<Mutex<Callable>>,
    pub return_type: TypeId,
}

impl YarnCallable {
    pub fn from_callable(callable: Callable, return_type: VariantType) -> Result<Self, String> {
        return match return_type {
            VariantType::BOOL => Ok(Self{callable: Arc::new(Mutex::new(callable)), return_type: YarnValue::Boolean(true).type_id()}),
            VariantType::INT => Ok(Self{callable: Arc::new(Mutex::new(callable)), return_type: YarnValue::Number(0.0).type_id()}),
            VariantType::FLOAT => Ok(Self{callable: Arc::new(Mutex::new(callable)), return_type: YarnValue::Number(0.0).type_id()}),
            VariantType::STRING => Ok(Self{callable: Arc::new(Mutex::new(callable)), return_type: YarnValue::String("".to_string()).type_id()}),
            _ => Err(format!("YarnCallable::from_callable return_type {:?} is not supported", return_type))
        }
    }
}

unsafe impl Send for YarnCallable {}
unsafe impl Sync for YarnCallable {}

impl YarnFn<fn(VariantArray) -> YarnCallableVariant> for YarnCallable {
    type Out = YarnCallableVariant;

    fn call(&self, input: Vec<YarnValue>) -> Self::Out {
        let callable = self.callable.lock().unwrap();
        let mut args = array![];
        for arg in input {
            args.push(YarnConversionUtils::yarn_value_to_variant(&arg));
        }
        let result = callable.callv(args);
        return YarnCallableVariant(result);
    }

    fn parameter_types(&self) -> Vec<TypeId> {
        let mut ids = vec![];
        let callable = self.callable.lock().unwrap();
        for argument in callable.as_inner().get_bound_arguments().iter_shared() {
            ids.push(YarnCallableVariant(argument).type_id());
        }
        return ids;
    }

    fn return_type(&self) -> TypeId {
        return self.return_type;
    }
}

#[derive(Clone)]
pub struct YarnCallableVariant(Variant);

impl IntoYarnValueFromNonYarnValue for YarnCallableVariant {
    fn into_yarn_value(self) -> YarnValue {
        return YarnConversionUtils::variant_to_yarn_value(&self.0).unwrap();
    }
}