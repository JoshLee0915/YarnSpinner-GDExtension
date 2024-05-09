use std::collections::HashMap;
use std::str::FromStr;
use godot::engine::utilities::push_error;
use godot::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use yarnspinner::prelude::YarnCompiler;

#[derive(Serialize, Deserialize)]
struct LocalizationInfo {
    #[serde(rename = "assets")]
    pub assets: String,
    #[serde(rename = "strings")]
    pub strings: String,
}

#[derive(Serialize, Deserialize)]
struct Project {
    #[serde(rename = "projectFileVersion")]
    pub file_version: i32,
    #[serde(rename = "sourceFiles")]
    pub source_file_patterns: Vec<String>,
    #[serde(rename = "excludeFiles")]
    pub exclude_file_patterns: Vec<String>,
    #[serde(rename = "localisation")]
    pub localization: HashMap<String, LocalizationInfo>,
    #[serde(rename = "baseLanguage")]
    pub base_language: String,
    #[serde(rename = "definitions")]
    pub definitions: String,
    #[serde(rename = "compilerOptions")]
    pub compiler_options: HashMap<String, Value>,
    #[serde(skip_serializing)]
    pub source_files: Vec<String>,
    #[serde(skip_serializing)]
    pub definitions_path: String,
}

impl Default for Project {
    fn default() -> Self {
        return Project{
            file_version: 2,
            source_file_patterns: vec!["**/*.yarn".to_string()],
            exclude_file_patterns: vec![],
            localization: Default::default(),
            base_language: "en".to_string(), // TODO: Find better way to set this instead of just defaulting to english
            definitions: "".to_string(),
            compiler_options: Default::default(),
            source_files: vec![],
            definitions_path: "".to_string(),
        }
    }
}

#[derive(GodotClass)]
#[class(tool, base=Resource)]
struct YarnProject {
    base: Base<Resource>,
    project: Project,

    #[export]
    pub last_import_had_implicit_string_ids: bool,
    #[export]
    pub last_import_had_any_strings: bool,
    #[export]
    pub is_successfully_parsed: bool,
    #[export]
    pub import_path: GString,
}

#[godot_api]
impl YarnProject {
    #[func]
    pub fn to_json(&self) -> Variant {
        return match serde_json::to_string(&self.project) {
            Ok(json) => {
                GString::from_str(json.as_str()).unwrap().to_variant()
            }
            Err(err) => {
                push_error(GString::from_str(err.to_string().as_str()).unwrap().to_variant(), &[]);
                Variant::nil()
            }
        }
    }
}

#[godot_api]
impl IResource for YarnProject {
    fn init(base: Base<Resource>) -> Self {
        return Self {
            base,
            project: Default::default(),
            last_import_had_implicit_string_ids: false,
            last_import_had_any_strings: false,
            is_successfully_parsed: false,
            import_path: Default::default(),
        }
    }
}
