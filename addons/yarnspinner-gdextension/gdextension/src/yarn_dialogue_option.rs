use godot::builtin::GString;
use godot::prelude::{Gd, godot_api, GodotClass, ToGodot};
use yarnspinner::prelude::DialogueOption;
use crate::yarn_line::YarnLine;

#[derive(GodotClass)]
#[class(init)]
pub struct YarnDialogueOption {
    pub option: Option<DialogueOption>,
}

#[godot_api]
impl YarnDialogueOption {
    pub fn new(option: &DialogueOption) -> Gd<Self> {
        return Gd::from_object(Self{
            option: Some(option.clone())
        });
    }

    #[func]
    pub fn get_id(&self) -> u64 {
        return self.option.as_ref().unwrap().id.0 as u64;
    }

    #[func]
    pub fn get_line(&self) -> Gd<YarnLine> {
        return YarnLine::new(&self.option.as_ref().unwrap().line);
    }

    #[func]
    pub fn get_destination_node(&self) -> GString {
        return self.option.as_ref().unwrap().destination_node.to_godot();
    }

    #[func]
    pub fn is_available(&self) -> bool {
        return self.option.as_ref().unwrap().is_available;
    }
}