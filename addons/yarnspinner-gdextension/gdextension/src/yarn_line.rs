use godot::prelude::*;
use yarnspinner::prelude::MarkupValue;
use yarnspinner::runtime::{Line, MarkupAttribute};

#[derive(GodotClass)]
#[class(init)]
pub struct YarnMarkupAttribute {
    pub(crate) attribute: Option<MarkupAttribute>,
}

#[godot_api]
impl YarnMarkupAttribute {
    pub fn new(attribute: &MarkupAttribute) -> Gd<Self> {
        return Gd::from_object(Self{ attribute: Some(attribute.clone()) })
    }

    #[func]
    pub fn get_name(&self) -> Variant {
        return match &self.attribute {
            None => Variant::nil(),
            Some(attribute) => Variant::from(attribute.name.to_godot()),
        }
    }

    #[func]
    pub fn get_property(&self, name: GString) -> Variant {
        return match &self.attribute {
            None => Variant::nil(),
            Some(attribute) => {
                return match attribute.property(name.to_string().as_str()) {
                    None => Variant::nil(),
                    Some(property) => Self::markup_value_to_variant(property),
                }
            }
        }
    }

    #[func]
    pub fn get_properties(&self) -> Dictionary {
        return match &self.attribute {
            None => dict! {},
            Some(attribute) => {
                let mut prop_dict = dict! {};
                for (key, value) in &attribute.properties {
                    prop_dict.set(key.to_variant(), Self::markup_value_to_variant(value));
                }
                return prop_dict;
            }
        };
    }

    fn markup_value_to_variant(value: &MarkupValue) -> Variant {
        return match value {
            MarkupValue::Integer(value) => Variant::from(*value),
            MarkupValue::Float(value) => Variant::from(*value),
            MarkupValue::String(value) => Variant::from(value.clone()),
            MarkupValue::Bool(value) => Variant::from(*value),
        }
    }
}

#[derive(GodotClass)]
#[class(init)]
pub struct YarnLine {
    line: Option<Line>,
}

#[godot_api]
impl YarnLine {
    pub fn new(line: &Line) -> Gd<Self> {
        return Gd::from_object(Self {
                line: Some(line.clone()),
            });
    }

    #[func]
    pub fn get_line_id(&self) -> Variant {
        return match &self.line {
            None => Variant::nil(),
            Some(line) => Variant::from(line.id.0.to_godot()),
        }
    }

    #[func]
    pub fn get_attribute(&self, name: GString) -> Variant {
        return match &self.line {
            None => Variant::nil(),
            Some(line) => { 
                return match line.attribute(name.to_string().as_str()) {
                    None => Variant::nil(),
                    Some(attribute) => YarnMarkupAttribute::new(attribute).to_variant(),
                }
            }
        }
    }

    #[func]
    pub fn get_character_name(&self) -> Variant {
        return match &self.line {
            None => Variant::nil(),
            Some(line) => {
                match line.character_name() {
                    None => Variant::nil(),
                    Some(name) => Variant::from(name.to_godot()),
                }
            }
        }
    }

    #[func]
    pub fn get_line(&self) -> Variant {
        return match &self.line {
            None => Variant::nil(),
            Some(line) => Variant::from(line.text_without_character_name().to_godot()),
        }
    }

    #[func]
    pub fn get_raw_text(&self) -> Variant {
        return match &self.line {
            None => Variant::nil(),
            Some(line) => Variant::from(line.text.to_godot()),
        }
    }

    #[func]
    pub fn get_text_for_attribute(&self, attribute: Gd<YarnMarkupAttribute>) -> Variant {
        return match &self.line {
            None => Variant::nil(),
            Some(line) => {
                return match &attribute.bind().attribute {
                    None => Variant::nil(),
                    Some(attr) => Variant::from(line.text_for_attribute(attr)),
                }
            }
        }
    }
}