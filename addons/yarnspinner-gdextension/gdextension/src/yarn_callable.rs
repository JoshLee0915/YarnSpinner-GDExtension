use crate::yarn_conversion_utils::YarnConversionUtils;
use godot::builtin::{array, Callable, Variant};
use godot::prelude::{VariantArray, VariantType};
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use godot::global::push_warning;
use godot::meta::ToGodot;
use godot::obj::EngineEnum;
use yarnspinner::core::YarnValue;
use yarnspinner::prelude::{IntoYarnValueFromNonYarnValue, YarnFn};

#[derive(Clone)]
pub struct YarnCallable
{
    pub callable: Arc<Mutex<Callable>>,
    pub return_type: TypeId,
    pub parameters: Vec<TypeId>,
}

impl YarnCallable {
    pub fn from_callable(callable: Callable, return_type: VariantType, parameters: &Vec<VariantType>) -> Result<Self, String> {
        let parameter_types = parameters.iter().map(|p| match p {
            &VariantType::BOOL => YarnValue::Boolean(true).type_id(),
            &VariantType::INT => YarnValue::Number(0.0).type_id(),
            &VariantType::FLOAT => YarnValue::Number(0.0).type_id(),
            &VariantType::STRING => YarnValue::String(String::new()).type_id(),
            &t => {
                push_warning(&[format!("Type {} is not valid for a parameter, defaulting to string", t.as_str()).to_variant()]);
                return YarnValue::String(String::new()).type_id();
            },
        }).collect::<Vec<TypeId>>();

        return match return_type {
            VariantType::BOOL => Ok(Self{callable: Arc::new(Mutex::new(callable)), return_type: YarnValue::Boolean(true).type_id(), parameters: parameter_types }),
            VariantType::INT => Ok(Self{callable: Arc::new(Mutex::new(callable)), return_type: YarnValue::Number(0.0).type_id(), parameters: parameter_types }),
            VariantType::FLOAT => Ok(Self{callable: Arc::new(Mutex::new(callable)), return_type: YarnValue::Number(0.0).type_id(), parameters: parameter_types }),
            VariantType::STRING => Ok(Self{callable: Arc::new(Mutex::new(callable)), return_type: YarnValue::String("".to_string()).type_id(), parameters: parameter_types }),
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
            args.push(&YarnConversionUtils::yarn_value_to_variant(&arg));
        }
        let result = callable.callv(&args);
        return YarnCallableVariant(result);
    }

    fn parameter_types(&self) -> Vec<TypeId> {
        return self.parameters.clone();
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