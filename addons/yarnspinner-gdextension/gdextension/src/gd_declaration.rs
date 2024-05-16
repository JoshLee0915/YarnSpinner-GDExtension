use std::error::Error;
use godot::engine::ProjectSettings;
use godot::prelude::*;
use yarnspinner::compiler::Declaration;
use yarnspinner::core::{Type, YarnValue};

#[derive(GodotConvert, Var, Export, Default)]
#[godot(via = GString)]
pub enum YarnBuiltInTypes {
    #[default]
    String,
    Boolean,
    Number
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct GDDeclaration {
    base: Base<Resource>,

    #[export]
    pub name: GString,
    #[export]
    pub yarn_type: YarnBuiltInTypes,
    #[export]
    pub default_value_bool: bool,
    #[export]
    pub default_value_number: f32,
    #[export]
    pub default_value_string: GString,
    #[export]
    pub description: GString,
    #[export]
    pub is_implicit: bool,
    #[export]
    pub source_yarn_asset_path: GString,
}

impl GDDeclaration {
    pub fn from_declaration(declaration: Declaration) -> Result<Gd<GDDeclaration>, Box<dyn Error>> {
        let mut gd_declaration = GDDeclaration::new_gd();
        let mut decl_bind = gd_declaration.bind_mut();

        decl_bind.name = declaration.name.to_godot();
        decl_bind.description = declaration.description.unwrap_or("".to_string()).to_godot();
        decl_bind.is_implicit = declaration.is_implicit;
        decl_bind.source_yarn_asset_path = ProjectSettings::singleton().localize_path(declaration.source_file_name.to_string().to_godot());

        return match declaration.r#type {
            Type::Boolean => {
                decl_bind.yarn_type = YarnBuiltInTypes::Boolean;
                if let YarnValue::Boolean(value) = declaration.default_value.unwrap_or(YarnValue::Boolean(false)) {
                    decl_bind.default_value_bool = value;
                }
                drop(decl_bind);
                Ok(gd_declaration.clone())
            }
            Type::Number => {
                decl_bind.yarn_type = YarnBuiltInTypes::Number;
                if let YarnValue::Number(value) = declaration.default_value.unwrap_or(YarnValue::Number(0.0)) {
                    decl_bind.default_value_number = value;
                }
                drop(decl_bind);
                Ok(gd_declaration.clone())
            }
            Type::String => {
                decl_bind.yarn_type = YarnBuiltInTypes::String;
                if let YarnValue::String(value) = declaration.default_value.unwrap_or(YarnValue::String("".to_string())) {
                    decl_bind.default_value_string = value.to_godot();
                }
                drop(decl_bind);
                Ok(gd_declaration.clone())
            }
            _ => {
                Err(format!("Invalid declaration type {}", decl_bind.name).into())
            }
        }
    }
}