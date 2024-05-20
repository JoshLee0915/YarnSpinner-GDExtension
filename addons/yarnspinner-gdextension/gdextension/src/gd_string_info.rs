use godot::prelude::*;
use yarnspinner::compiler::StringInfo;

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct GDStringInfo {
    base: Base<Object>,
    pub text: GString,
    pub node_name: GString,
    pub line_number: u32,
    pub file_name: GString,
    pub is_implicit_tag: bool,
    pub metadata: Array<GString>,
}

impl GDStringInfo {
    pub fn from_string_info(string_info: &StringInfo) -> Gd<GDStringInfo> {
        let mut metadata = array![];
        for m in &string_info.metadata {
            metadata.push(m.to_godot())
        }

        return Gd::from_init_fn(|base|{
            return Self{
                base,
                text: string_info.text.to_godot(),
                node_name: string_info.node_name.to_godot(),
                line_number: u32::try_from(string_info.line_number).unwrap(),
                file_name: string_info.file_name.to_godot(),
                is_implicit_tag: string_info.is_implicit_tag,
                metadata,
            };
        })
    }
}