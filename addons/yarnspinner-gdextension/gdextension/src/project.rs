use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use glob::glob;
use godot::engine::ProjectSettings;
use godot::engine::utilities::push_error;
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::function_info::FunctionInfo;
use crate::gd_declaration::GDDeclaration;
use crate::localization::Localization;

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
    pub source_files: Vec<String>,
    #[serde(rename = "excludeFiles")]
    pub exclude_file_patterns: Vec<String>,
    #[serde(rename = "localisation")]
    pub localization: HashMap<String, LocalizationInfo>,
    #[serde(rename = "baseLanguage")]
    pub base_language: String,
    #[serde(rename = "definitions")]
    pub definitions: Option<String>,
    #[serde(rename = "compilerOptions")]
    pub compiler_options: HashMap<String, Value>,
}

impl Default for Project {
    fn default() -> Self {
        return Project{
            file_version: 2,
            source_files: vec!["**/*.yarn".to_string()],
            exclude_file_patterns: vec![],
            localization: Default::default(),
            base_language: "en".to_string(), // TODO: Find better way to set this instead of just defaulting to english
            definitions: None,
            compiler_options: Default::default(),
        }
    }
}

#[derive(GodotClass)]
#[class(tool, init, base=Object)]
pub struct YarnLocalizationInfo {
    base: Base<Object>,
    #[export]
    pub assets: GString,
    #[export]
    pub strings: GString,
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct YarnProjectError {
    base: Base<Resource>,
    #[export]
    pub file_name: GString,
    #[export]
    pub message: GString,
    #[export]
    pub context: GString,
}

#[derive(GodotClass)]
#[class(tool, base=Resource)]
pub struct YarnProject {
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
    #[export]
    pub json_project_path: GString,
    #[export]
    pub project_errors: Array<Gd<YarnProjectError>>,
    #[export]
    pub declarations: Array<Gd<GDDeclaration>>,
    #[export]
    pub list_of_functions: Array<Gd<FunctionInfo>>,
    #[export]
    pub compiled_yarn_program_json: GString,
    #[export]
    pub base_localization: Option<Gd<Localization>>,
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
                push_error(&[GString::from_str(err.to_string().as_str()).unwrap().to_variant()]);
                Variant::nil()
            }
        }
    }

    #[func]
    pub fn load_from_file(&mut self, file: GString) {
        // TODO: Better error handling
        let path = ProjectSettings::singleton().globalize_path(file.clone()).to_string();
        let project_file = fs::read_to_string(&path).expect(&format!("Failed to load {}", &path));
        let project = serde_json::from_str::<Project>(&project_file).expect("Failed to serialize json to Project type");
        if project.file_version != 2 {
            push_error(&[format!("Project file at {} has incorrect file version (expected {}, got {})", file, 2,  self.project.file_version).to_variant()]);
            return;
        }
        self.project = project;
        self.json_project_path = path.to_godot();
    }

    #[func]
    pub fn get_base_language(&self) -> GString {
        return self.project.base_language.to_godot();
    }

    #[func]
    pub fn set_base_language(&mut self, base_language: GString) {
        self.project.base_language = base_language.to_string();
    }

    #[func]
    pub fn save_project(&self) {
        let project_path = &self.json_project_path;
        self.save_to_file(ProjectSettings::singleton().globalize_path(project_path.clone()));
    }

    #[func]
    pub fn save_to_file(&self, path: GString) {
        // TODO: Better error handling
        let json = serde_json::to_string(&self.project).expect("Failed to serialize project to json");
        fs::write(path.to_string(), json).expect("Failed to save file");
    }

    #[func]
    pub fn get_source_files(&self) -> Array<GString> {
        let json_path_string = self.json_project_path.to_string();
        let json_path = Path::new(&json_path_string);
        let working_dir = match json_path.parent() {
            None => "".to_string(),
            Some(value) => format!("{}/", value.to_str().unwrap().to_string())
        };

        let mut exclude = Array::<GString>::new();
        for pattern in &self.project.exclude_file_patterns {
            let full_pattern = format!("{}{}", working_dir, pattern);
            for entry in glob(&full_pattern).expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => {
                        exclude.push(GString::from_str(path.to_str().unwrap()).unwrap())
                    }
                    Err(err) => { panic!("{}", err) }
                }
            }
        }

        let mut source_files = Array::<GString>::new();
        for pattern in &self.project.source_files {
            let full_pattern = format!("{}{}", working_dir, pattern);
            for entry in glob(&full_pattern).expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => {
                        let file_path = GString::from_str(path.to_str().unwrap()).unwrap();
                        if !exclude.contains(&file_path) {
                            source_files.push(file_path);
                        }
                    }
                    Err(err) => { panic!("{}", err) }
                }
            }
        }
        return source_files;
    }

    #[func]
    pub fn get_default_json_project_path(&self) -> GString {
        return self.base().get_path().to_string().replace(".tres", ".yarnproject").to_godot();
    }

    #[func]
    pub fn get_source_file_patterns(&self) -> Array<GString> {
        let mut patterns = Array::<GString>::new();
        for source_file in &self.project.source_files {
            patterns.push(source_file.to_godot())
        }
        return patterns;
    }

    #[func]
    pub fn add_source_file_pattern(&mut self, pattern: GString) -> Array<GString> {
        if self.project.source_files.contains(&pattern.to_string()) {
            self.project.source_files.push(pattern.to_string());
        }
        return self.get_source_files();
    }

    #[func]
    pub fn remove_source_file_pattern(&mut self, pattern: GString) -> Array<GString> {
        let mut new_source_file_list = vec![];
        for source_file in &self.project.source_files {
            if !source_file.eq(&pattern.to_string()) {
                new_source_file_list.push(source_file.clone());
            }
        }
        self.project.source_files = new_source_file_list;
        return self.get_source_files();
    }
    
    #[func]
    pub fn get_localization(&self) -> Dictionary {
        let mut localization = Dictionary::new();
        for (key, localization_info) in &self.project.localization {
            localization.set(key.to_godot(), Gd::from_init_fn(|base| {
                return YarnLocalizationInfo{
                    base,
                    assets: localization_info.assets.to_godot(),
                    strings: localization_info.strings.to_godot(),
                }
            }));
        }
        return localization;
    }

    #[func]
    pub fn localization_contains_key(&self, locale_code: GString) -> bool {
        return self.project.localization.contains_key(&locale_code.to_string());
    }

    #[func]
    pub fn set_localization(&mut self, locale_code: GString, localization_info: Gd<YarnLocalizationInfo>) {
        self.project.localization.insert(locale_code.to_string(), LocalizationInfo{
            assets: localization_info.bind().assets.to_string(),
            strings: localization_info.bind().strings.to_string(),
        });
    }

    #[func]
    pub fn remove_localization(&mut self, locale_code: GString) {
        self.project.localization.remove(&locale_code.to_string());
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
            json_project_path: Default::default(),
            project_errors: array![],
            declarations: array![],
            list_of_functions: array![],
            compiled_yarn_program_json: Default::default(),
            base_localization: None,
        }
    }
}
