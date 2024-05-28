use godot::prelude::*;
use yarnspinner::core::YarnValue;
use yarnspinner::prelude::DialogueError;
use yarnspinner::runtime::VariableStorageError;
use yarnspinner::runtime::VariableStorageError::InternalError;
use crate::dialogue_runner::YarnDialogueResult;

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
        return match value {
            YarnValue::Number(v) => v.to_variant(),
            YarnValue::String(v) => v.to_variant(),
            YarnValue::Boolean(v) => v.to_variant(),
        }
    }

    pub fn variant_to_yarn_value(value: &Variant) -> Result<YarnValue, VariableStorageError> {
        return match value.get_type() {
            VariantType::BOOL => Ok(YarnValue::Boolean(value.to())),
            VariantType::INT => Ok(YarnValue::Number(value.to())),
            VariantType::FLOAT => Ok(YarnValue::Number(value.to())),
            VariantType::STRING => Ok(YarnValue::String(value.to_string())),
            _ => Err(InternalError { error: format!("Failed to convert {} to a yarn value", value.clone()).into() }),
        }
    }
}