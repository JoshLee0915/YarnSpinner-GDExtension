use godot::prelude::*;
use godot::engine::Resource;

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
struct MarkupPalette {
    base: Base<Resource>,
    #[export]
    pub colour_markers: Dictionary
}

#[godot_api]
impl MarkupPalette {
    #[func]
    pub fn color_for_marker(&self, marker: GString) -> Variant {
        return match self.colour_markers.get(marker) {
            Some(value) => {
                if value.get_type() == VariantType::COLOR {
                    return value;
                }
                return Variant::nil()
            },
            _ => Variant::nil()
        }
    }
}