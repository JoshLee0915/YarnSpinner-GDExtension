use crate::dialogue_runner::YarnDialogueResult;
use godot::prelude::*;
use yarnspinner::core::YarnValue;
use yarnspinner::prelude::DialogueError;
use yarnspinner::runtime::VariableStorageError;
use yarnspinner::runtime::VariableStorageError::InternalError;

pub struct YarnConversionUtils {}

impl YarnConversionUtils {
    pub fn yarn_dialogue_error_to_yarn_dialogue_result(error: &DialogueError) -> YarnDialogueResult {
        return match error {
            DialogueError::MarkupParseError(_) => YarnDialogueResult::MarkupParseError,
            DialogueError::LineProviderError { .. } => YarnDialogueResult::LineProviderError,
            DialogueError::InvalidOptionIdError { .. } => YarnDialogueResult::InvalidOptionIdError,
            DialogueError::UnexpectedOptionSelectionError => YarnDialogueResult::UnexpectedOptionSelectionError,
            DialogueError::ContinueOnOptionSelectionError => YarnDialogueResult::ContinueOnOptionSelectionError,
            DialogueError::NoNodeSelectedOnContinue => YarnDialogueResult::NoNodeSelectedOnContinue,
            DialogueError::InvalidNode { .. } => YarnDialogueResult::InvalidNode,
            DialogueError::VariableStorageError(_) => YarnDialogueResult::VariableStorageError,
        }
    }

    pub fn yarn_value_to_variant(value: &YarnValue) -> Variant {
        if let Ok(v) = f32::try_from(value) {
            return v.to_variant()
        }
        if let Ok(v) = bool::try_from(value) {
            return v.to_variant()
        }
        return String::try_from(value).expect("Yarn value should always be convertable to string").to_variant();
    }

    pub fn variant_to_yarn_value(value: &Variant) -> Result<YarnValue, VariableStorageError> {
        return match value.get_type() {
            VariantType::BOOL => Ok(YarnValue::Boolean(value.to())),
            VariantType::INT => {
                let v: i32 = value.to();
                return Ok(YarnValue::Number(v as f32));
            },
            VariantType::FLOAT => Ok(YarnValue::Number(value.to())),
            VariantType::STRING => Ok(YarnValue::String(value.to_string())),
            _ => Err(InternalError { error: format!("Failed to convert {} with type {} to a yarn value", value.stringify(), value.get_type().to_variant()).into() }),
        }
    }
}