use godot::prelude::*;
use crate::string_table_entry::GDStringTableEntry;

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct Localization {
    base: Base<Resource>,
    #[export]
    pub local_code: GString,
    #[export]
    pub string_table: Dictionary,
    #[export]
    pub string_file: GString,
    runtime_string_table: Dictionary,
}

#[godot_api]
impl Localization {
    #[func]
    pub fn get_localized_string(&self, key: GString) -> Variant {
        if let Some(value) = self.runtime_string_table.get(key.clone()) {
            return value;
        }

        if let Some(value) = self.string_table.get(key.clone()) {
            let string_table_entry: Gd<GDStringTableEntry> = value.to();
            return string_table_entry.bind().text.to_variant();
        }

        return Variant::nil();
    }

    #[func]
    pub fn get_string_table_entries(&self) -> Array<Gd<GDStringTableEntry>> {
        let mut entries = array![];
        for value in self.string_table.values_array().iter_shared() {
            let entry: Gd<GDStringTableEntry> = value.to();
            entries.push(entry);
        }
        return entries;
    }

    #[func]
    pub fn contains_localized_string(&self, key: GString) -> bool {
        return self.runtime_string_table.contains_key(key.clone()) || self.string_table.contains_key(key.clone());
    }

    #[func]
    pub fn add_localized_string_to_asset(&mut self, key: GString, value: Gd<GDStringTableEntry>) {
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
}