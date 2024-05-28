use std::collections::HashMap;
use godot::global::{push_warning};
use godot::prelude::*;
use yarnspinner::core::LineId;
use yarnspinner::prelude::StringInfo;

#[derive(GodotClass, Debug)]
#[class(tool, init, base=Resource)]
pub struct Localization {
    base: Base<Resource>,
    #[export]
    pub local_code: GString,
    #[export]
    string_table: Dictionary,
    #[var]
    runtime_string_table: Dictionary,
}

impl Localization {
    pub fn new(local_code: &str, string_table: HashMap<LineId, StringInfo>) -> Gd<Self> {
        let mut table = dict! {};
        for (line_id, string_info) in string_table {
            table.set(line_id.0.to_godot(), string_info.text.to_godot());
        }
        return Gd::from_init_fn(|base| {
            return Self{
                base,
                local_code: local_code.to_godot(),
                string_table: table.clone(),
                runtime_string_table: Default::default(),
            }
        })
    }
}

#[godot_api]
impl Localization {
    #[func]
    pub fn extend_runtime_table(&mut self, localization: Gd<Localization>) {
        let bound_localization = localization.bind();
        if self.local_code != bound_localization.local_code {
            push_warning(&[format!("Local code {} does not match the passed local of {}", self.local_code.clone(), bound_localization.local_code.clone()).to_variant()]);
            return;
        }
        self.runtime_string_table.extend_dictionary(bound_localization.string_table.clone(), false)
    }

    #[func]
    pub fn get_localized_string(&self, key: GString) -> Variant {
        if let Some(value) = self.runtime_string_table.get(key.clone()) {
            return value;
        }

        if let Some(value) = self.string_table.get(key.clone()) {
            let string_table_entry: GString = value.to();
            return string_table_entry.to_variant();
        }

        return Variant::nil();
    }

    #[func]
    pub fn get_string_table_entries(&self) -> Array<GString> {
        let mut entries = array![];
        for value in self.string_table.values_array().iter_shared() {
            let entry: GString = value.to();
            entries.push(entry);
        }
        return entries;
    }

    #[func]
    pub fn contains_localized_string(&self, key: GString) -> bool {
        return self.runtime_string_table.contains_key(key.clone()) || self.string_table.contains_key(key.clone());
    }

    #[func]
    pub fn add_localized_string_to_asset(&mut self, key: GString, value: GString) {
        self.string_table.set(key, value);
    }

    #[func]
    pub fn add_localized_string(&mut self, key: GString, value: GString) {
        self.runtime_string_table.set(key, value);
    }

    #[func]
    pub fn clear(&mut self) {
        self.string_table.clear();
        self.runtime_string_table.clear();
    }

    #[func]
    pub fn get_line_ids(&self) -> VariantArray {
        let mut keys = array![];
        keys.extend_array(self.runtime_string_table.keys_array());
        keys.extend_array(self.string_table.keys_array());
        return keys;
    }

    #[func]
    pub fn generate_localization_csv(localizations: VariantArray) -> GString {
        let mut header = vec!["keys".to_string()];
        for localization_variant in localizations.iter_shared() {
            let localization: Gd<Localization> = localization_variant.to();
            header.push(localization.bind().local_code.to_string());
        }

        let mut translations = HashMap::<String, Vec<String>>::new();
        for localization_variant in localizations.iter_shared() {
            let localization: Gd<Localization> = localization_variant.to();
            let local_code = localization.bind().local_code.to_string();
            let index = header.iter().position(|code| *code == local_code).unwrap();
            for (key, value) in localization.bind().string_table.iter_shared() {
                let local_key = key.to_string();
                if !translations.contains_key(&local_key) {
                    let mut strings = Vec::<String>::with_capacity(header.len());
                    strings.resize(header.len(), "NO_TRANSLATION_PROVIDED".to_string());
                    strings[0] = local_key.clone();
                    translations.insert(local_key.clone(), strings);
                }
                translations.get_mut(&local_key).unwrap()[index] = value.to_string();
            }
        }

        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .delimiter(b',')
            .from_writer(vec![]);

        writer.write_record(&header).expect(&format!("Failed to write header {} to translation csv", header.join(",").as_str()));
        for translation_row in translations.values() {
            writer.write_record(translation_row).expect(&format!("Failed to write {} to translation csv", translation_row.join(",").as_str()))
        }

        return String::from_utf8(writer.into_inner().unwrap()).unwrap().to_godot();
    }

    #[func]
    pub fn parse_from_csv(csv_text: GString) -> Array<Gd<Localization>> {
        let string_text = csv_text.to_string();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b',')
            .from_reader(string_text.as_bytes());


        let mut localizations = HashMap::<&str, Gd<Localization>>::new();
        let mut local_codes = vec!["keys"];
        let header = reader.records().next().unwrap().unwrap();
        for local_code in &header {
            if local_code == "keys" {
                continue;
            }
            localizations.insert(local_code, Localization::new(local_code, HashMap::new()));
            local_codes.push(local_code);
        }

        while let Some(row) = reader.records().next() {
            let row_data = row.expect("Failed to parse csv record");
            let key = &row_data[0];
            for (index, data) in row_data.iter().enumerate() {
                if data == key {
                    continue;
                }
                let local_code = local_codes[index];
                let localization = &mut localizations.get_mut(local_code).unwrap().bind_mut();
                localization.add_localized_string_to_asset(key.to_godot(), data.to_godot());
            }
        }

        let mut result = array![];
        for value in localizations.values() {
            result.push(value.clone());
        }
        return result;
    }
}