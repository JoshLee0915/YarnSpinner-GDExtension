mod player;
mod markup_palette;
mod project;

use godot::prelude::*;

struct YarnSpinner;

#[gdextension]
unsafe impl ExtensionLibrary for YarnSpinner {}