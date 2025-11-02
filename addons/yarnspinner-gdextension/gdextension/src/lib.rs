mod markup_palette;
mod project;
mod function_info;
mod yarn_compiler;
mod gd_declaration;
mod gd_string_info;
mod gd_compilation;
mod localization;
mod dialogue_runner;
mod yarn_variable_storage;
mod translation_server_text_provider;
mod yarn_line;
mod yarn_dialogue_option;
mod yarn_conversion_utils;
mod yarn_callable;

use godot::classes::Engine;
use godot::prelude::*;
use crate::yarn_compiler::{YARN_COMPILER_SINGLETON_NAME, YarnCompilerSingleton};

struct YarnSpinner;

#[gdextension]
unsafe impl ExtensionLibrary for YarnSpinner {
    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Core => {}
            InitLevel::Servers => {}
            InitLevel::Scene => {
                Engine::singleton().register_singleton(
                    YARN_COMPILER_SINGLETON_NAME,
                    &YarnCompilerSingleton::new_alloc(),
                );
            }
            InitLevel::Editor => {}
        }
    }

    fn on_level_deinit(level: InitLevel) {
        match level {
            InitLevel::Core => {}
            InitLevel::Servers => {}
            InitLevel::Scene => {
                let mut engine = Engine::singleton();
                let singleton = engine
                    .get_singleton(YARN_COMPILER_SINGLETON_NAME)
                    .expect("cannot retrieve the singleton");

                engine.unregister_singleton(YARN_COMPILER_SINGLETON_NAME);
                singleton.free();
            }
            InitLevel::Editor => {}
        }
    }
}