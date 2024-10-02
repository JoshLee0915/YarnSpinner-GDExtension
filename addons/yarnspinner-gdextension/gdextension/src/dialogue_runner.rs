use std::collections::HashMap;

use godot::engine::utilities::{push_error, push_warning};
use godot::prelude::*;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use yarnspinner::core::{Library, Program};
use yarnspinner::prelude::{Dialogue, DialogueEvent, YarnLibrary};
use crate::localization::Localization;

use crate::project::YarnProject;
use crate::translation_server_text_provider::TranslationServerTextProvider;
use crate::yarn_callable::YarnCallable;
use crate::yarn_conversion_utils::YarnConversionUtils;
use crate::yarn_dialogue_option::YarnDialogueOption;
use crate::yarn_line::YarnLine;
use crate::yarn_variable_storage::{VariableStorageWrapper, YarnVariableStorage};

#[derive(GodotConvert, Var, Export, Default, Debug)]
#[godot(via = GString)]
pub enum YarnDialogueResult {
    #[default]
    Ok,
    DialogueComplete,
    MarkupParseError,
    LineProviderError,
    InvalidOptionIdError,
    UnexpectedOptionSelectionError,
    ContinueOnOptionSelectionError,
    NoNodeSelectedOnContinue,
    InvalidNode,
    VariableStorageError,
    DialogueNotRunning,
    DialogueAlreadyRunning,
    DialogueRunnerNotSet,
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct DialogueRunner {
    base: Base<Node>,
    dialogue_runner: Option<Dialogue>,
    dialogue_running: bool,
    commands: HashMap<StringName, Callable>,
    #[var]
    pub current_line: Option<Gd<YarnLine>>,
    #[var]
    pub current_options: Array<Gd<YarnDialogueOption>>,
    #[export]
    pub yarn_variable_store: Option<Gd<YarnVariableStorage>>,
    #[export]
    pub projects: Array<Gd<YarnProject>>,
}


#[godot_api]
impl DialogueRunner {
    #[signal]
    fn node_start(node_name: GString) {}

    #[signal]
    fn node_complete(node_name: GString) {}

    #[signal]
    fn dialogue_start() {}

    #[signal]
    fn dialogue_complete() {}

    #[signal]
    fn next_line(line: Gd<YarnLine>) {}

    #[signal]
    fn next_line_hints(line_ids: Array<GString>) {}

    #[signal]
    fn options_available(options: Array<Gd<YarnDialogueOption>>) {}

    #[func]
    pub fn is_dialogue_running(&self) -> bool {
        return self.dialogue_running;
    }

    #[func]
    pub fn is_waiting_for_option_selection(&self) -> bool {
        match &self.dialogue_runner {
            None => false,
            Some(runner) => runner.is_waiting_for_option_selection(),
        }
    }

    #[func]
    pub fn node_exists(&self, node_name: GString) -> bool {
        return match &self.dialogue_runner {
            None => false,
            Some(runner) => {
                return runner.node_exists(node_name.to_string().as_str());
            }
        }
    }
    
    #[func]
    pub fn get_tags_for_node(&self, node_name: GString) -> Array<GString> {
        let mut tags = array![];
        match self.dialogue_runner.as_ref().unwrap().get_tags_for_node(node_name.to_string().as_str()) {
            None => push_warning(&[format!("Node {} not found", node_name).to_variant()]),
            Some(node_tags) => {
                for tag in node_tags {
                    tags.push(tag.to_godot());
                }
            }
        }
        return tags;
    }

    #[func]
    pub fn get_current_node(&self) -> Variant {
        if let Some(runner) = &self.dialogue_runner {
            return match runner.current_node() {
                None => Variant::nil(),
                Some(node_name) => node_name.to_variant(),
            }
        }
        return Variant::nil();
    }

    #[func]
    pub fn build(&mut self) {
        if let Some(runner) = &mut self.dialogue_runner {
            runner.unload_all();
            for project in self.projects.iter_shared() {
                let json = &project.bind().compiled_yarn_program_json;
                match serde_json::from_str::<Program>(json.to_string().as_str()) {
                    Ok(program) => {
                        runner.add_program(program);
                    }
                    Err(err) => {
                        push_error(&[format!("Error: {}. Failed to parse program: {}\n{}", err, &project.get_name(), &json).to_variant()])
                    }
                };
            }
        } else {
            push_warning(&["Dialogue runner not set".to_variant()]);
        }
    }

    #[func]
    pub fn start_dialogue(&mut self, node_name: GString) -> YarnDialogueResult {
        if self.dialogue_running {
            push_error(&[format!("Can't start dialogue from node {}: the dialogue is currently in the middle of running. Stop the dialogue first.", node_name).to_variant()]);
            return YarnDialogueResult::DialogueAlreadyRunning
        }

        if self.dialogue_runner.is_none() {
            return YarnDialogueResult::DialogueRunnerNotSet;
        }

        return match self.dialogue_runner.as_mut().unwrap().set_node(node_name) {
            Ok(_) => {
                self.dialogue_running = true;
                self.base_mut().emit_signal("dialogue_start".into(), &[]);
                self.continue_dialogue()
            },
            Err(err) => {
                push_error(&[err.to_string().to_variant()]);
                YarnConversionUtils::yarn_dialogue_error_to_yarn_dialogue_result(&err)
            },
        }
    }

    #[func]
    pub fn stop_dialogue(&mut self) -> YarnDialogueResult {
        if let Some(runner) = &mut self.dialogue_runner {
            let events = runner.stop();
            self.process_events(events);
        }
        return YarnDialogueResult::Ok;
    }

    #[func]
    pub fn clear(&mut self) {
        if let Some(runner) = &mut self.dialogue_runner {
            runner.unload_all();
            self.projects.clear();
        }
    }

    #[func]
    pub fn continue_dialogue(&mut self) -> YarnDialogueResult {
        if self.is_dialogue_running() {
            self.current_line = None;
            self.current_options.clear();
            let runner = self.dialogue_runner.as_mut().unwrap();
            return match runner.continue_() {
                Ok(events) => {
                    self.process_events(events);
                    if self.is_dialogue_running() {
                        return YarnDialogueResult::Ok;
                    } else {
                        return YarnDialogueResult::DialogueComplete;
                    }
                }
                Err(err) => {
                    push_error(&[err.to_string().to_variant()]);
                    YarnConversionUtils::yarn_dialogue_error_to_yarn_dialogue_result(&err)
                }
            }
        }
        return YarnDialogueResult::DialogueNotRunning;
    }

    #[func]
    pub fn select_option(&mut self, selection: Gd<YarnDialogueOption>) -> YarnDialogueResult {
        return match &mut self.dialogue_runner {
            None => YarnDialogueResult::DialogueRunnerNotSet,
            Some(runner) => {
                return match &selection.bind().option {
                    None => YarnDialogueResult::InvalidOptionIdError,
                    Some(option) => {
                        let result = runner.set_selected_option(option.id);
                        return match result {
                            Ok(_) => YarnDialogueResult::Ok,
                            Err(err) => YarnConversionUtils::yarn_dialogue_error_to_yarn_dialogue_result(&err),
                        }
                    },
                };
            }
        }
    }

    #[func]
    pub fn register_command(&mut self, command_name: GString, callable: Callable) {
        self.commands.insert(StringName::from(command_name), callable);
    }

    #[func]
    pub fn remove_command(&mut self, command_name: GString) -> bool {
        return self.commands.remove(&StringName::from(command_name)).is_some();
    }

    #[func]
    pub fn register_function(&mut self, function_name: GString, callable: Callable, return_type: i32) {
        if let Some(runner) = &mut self.dialogue_runner {
            match YarnCallable::from_callable(callable, VariantType{ord: return_type}) {
                Ok(callable) => {
                    runner.library_mut().add_function(function_name.to_string(), callable);
                },
                Err(err) => {panic!("{}", err)}
            }

        }
    }
}

impl DialogueRunner {
    fn process_events(&mut self, events: Vec<DialogueEvent>) {
        for event in events {
            match event {
                DialogueEvent::Line(line) => {
                    let yarn_line = YarnLine::new(&line);
                    self.base_mut().emit_signal(StringName::from("next_line"), &[yarn_line.to_variant()]);
                    self.current_line = Some(yarn_line.clone());
                }
                DialogueEvent::Options(options) => {
                    let mut dialogue_options = array![];
                    for option in options {
                        dialogue_options.push(YarnDialogueOption::new(&option));
                    }
                    self.base_mut().emit_signal(StringName::from("options_available"), &[dialogue_options.to_variant()]);
                    self.current_options.extend_array(dialogue_options);
                }
                DialogueEvent::Command(command) => {
                    if let Some(callable) = self.commands.get(&StringName::from(command.name.clone())) {
                        let mut parameters = array![];
                        for parameter in &command.parameters {
                            parameters.push(YarnConversionUtils::yarn_value_to_variant(parameter));
                        }
                        callable.callv(parameters);
                    } else {
                        push_warning(&[format!("Failed to find registered command '{}'", command.name.clone()).to_variant()]);
                    }
                }
                DialogueEvent::NodeComplete(node_name) => {
                    self.base_mut().emit_signal(StringName::from("node_complete"), &[node_name.to_variant()]);
                }
                DialogueEvent::NodeStart(node_name) => {
                    self.base_mut().emit_signal(StringName::from("node_start"), &[node_name.to_variant()]);
                }
                DialogueEvent::LineHints(hints) => {
                    let mut line_ids = array![];
                    for line_id in hints {
                        line_ids.push(line_id.0.to_variant());
                    }
                    self.base_mut().emit_signal(StringName::from("next_line_hints"), &[line_ids.to_variant()]);
                }
                DialogueEvent::DialogueComplete => {
                    self.base_mut().emit_signal(StringName::from("dialogue_complete"), &[]);
                    self.dialogue_running = false;
                }
            }
        }
    }

    fn build_library() -> Library {
        let mut library = YarnLibrary::standard_library();
        library
            .add_function("random", || SmallRng::from_entropy().gen_range(0.0..1.0))
            .add_function("random_range", |min: f32, max: f32| {
                if let Some(min) = min.as_int() {
                    if let Some(max_inclusive) = max.as_int() {
                        return SmallRng::from_entropy().gen_range(min..=max_inclusive) as f32;
                    }
                }
                SmallRng::from_entropy().gen_range(min..max)
            })
            .add_function("dice", |sides: u32| {
                if sides == 0 {
                    return 1;
                }
                SmallRng::from_entropy().gen_range(1..=sides)
            })
            .add_function("round", |num: f32| num.round() as i32)
            .add_function("round_places", |num: f32, places: u32| {
                num.round_places(places)
            })
            .add_function("floor", |num: f32| num.floor() as i32)
            .add_function("ceil", |num: f32| num.ceil() as i32)
            .add_function("inc", |num: f32| {
                if let Some(num) = num.as_int() {
                    num + 1
                } else {
                    num.ceil() as i32
                }
            })
            .add_function("dec", |num: f32| {
                if let Some(num) = num.as_int() {
                    num - 1
                } else {
                    num.floor() as i32
                }
            })
            .add_function("decimal", |num: f32| num.fract())
            .add_function("int", |num: f32| num.trunc() as i32);
        return library;
    }
}

#[godot_api]
impl INode for DialogueRunner {
    fn init(base: Base<Self::Base>) -> Self {
        return Self{
            base,
            dialogue_runner: None,
            dialogue_running: false,
            current_line: None,
            current_options: Default::default(),
            commands: Default::default(),
            yarn_variable_store: None,
            projects: Default::default(),
        }
    }

    fn ready(&mut self) {
        let store = VariableStorageWrapper::wrap(self.yarn_variable_store.as_ref().unwrap());
        let mut fallback_localization: Option<Gd<Localization>> = None;
        for project in self.projects.iter_shared() {
            if let Some(localization) = &mut fallback_localization {
                if project.bind().base_localization.is_some() {
                    localization.bind_mut().extend_runtime_table(project.bind().base_localization.as_ref().unwrap().clone());
                }
            } else {
                fallback_localization = project.bind().base_localization.clone();
            }
        }
        let mut runner = Dialogue::new(store, TranslationServerTextProvider::new(fallback_localization));
        runner.library_mut().extend(DialogueRunner::build_library());
        self.dialogue_runner = Some(runner);
        self.build();
    }
}


trait FloatExt: Copy {
    fn as_int(self) -> Option<i32>;
    fn round_places(self, places: u32) -> Self;
}

impl FloatExt for f32 {
    fn as_int(self) -> Option<i32> {
        (self.fract() <= f32::EPSILON).then_some(self as i32)
    }

    fn round_places(self, places: u32) -> Self {
        let factor = 10_u32.pow(places) as f32;
        (self * factor).round() / factor
    }
}