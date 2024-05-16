use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct FunctionInfo {
    base: Base<Resource>,

    #[export]
    pub name: GString,
    #[export]
    pub return_type: GString,
    #[export]
    pub parameters: Array<GString>
}