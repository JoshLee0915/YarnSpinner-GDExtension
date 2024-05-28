use std::any::Any;
use std::sync::{Arc, Mutex};

use godot::builtin::StringName;
use godot::engine::TranslationServer;
use godot::obj::Gd;
use godot::prelude::ToGodot;
use yarnspinner::core::LineId;
use yarnspinner::prelude::{Language, TextProvider};

use crate::localization::Localization;

#[derive(Debug)]
pub struct TranslationServerTextProvider {
    fallback_localization: Option<Arc<Mutex<Gd<Localization>>>>,
}

unsafe impl Send for TranslationServerTextProvider {}
unsafe impl Sync for TranslationServerTextProvider {}

impl TranslationServerTextProvider {
    pub fn new(fallback_localization: Option<Gd<Localization>>) -> Box<dyn TextProvider> {
        return match fallback_localization {
            None => Box::new(TranslationServerTextProvider{fallback_localization: None}),
            Some(localization) => Box::new(TranslationServerTextProvider{fallback_localization: Some(Arc::new(Mutex::new(localization)))})
        };
    }
}

impl TextProvider for TranslationServerTextProvider {
    fn accept_line_hints(&mut self, _line_ids: &[LineId]) {
        // no-op
    }

    fn get_text(&self, id: &LineId) -> Option<String> {
        let translated_line = TranslationServer::singleton().translate(StringName::from(&id.0));
        // If we get a string back that is the same as the ID attempt to use the fallback local if set
        if translated_line.to_string() == id.0 {
            return match &self.fallback_localization {
                None => None,
                Some(localization) => {
                    let fallback_local_guard = localization.lock().unwrap();
                    let fallback_local = fallback_local_guard.bind();

                    let set_language = self.get_language().unwrap().to_string();
                    let fallback_language = fallback_local.local_code.to_string();
                    if set_language != fallback_language || !fallback_local.contains_localized_string(id.0.to_godot()) {
                        return None;
                    }
                    return Some(fallback_local.get_localized_string(id.0.to_godot()).to_string());
                }
            }
        }
        return Some(translated_line.to_string());
    }

    fn set_language(&mut self, language: Option<Language>) {
        if let Some(locale) = language {
            TranslationServer::singleton().set_locale(locale.to_string().to_godot());
        }
    }

    fn get_language(&self) -> Option<Language> {
        let locale = TranslationServer::singleton().get_locale();
        return Some(Language::new(locale.to_string()));
    }

    fn are_lines_available(&self) -> bool {
        let locale = TranslationServer::singleton().get_locale();
        let translation = TranslationServer::singleton().get_translation_object(locale.clone());
        return match translation {
            None => {
                match &self.fallback_localization {
                    None => return false,
                    Some(localization) => localization.lock().unwrap().bind().local_code == locale,
                }
            },
            Some(_) => true,
        } ;
    }

    fn as_any(&self) -> &dyn Any {
        return self;
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        return self;
    }
}