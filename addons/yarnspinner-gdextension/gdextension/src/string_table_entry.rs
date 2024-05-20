use godot::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StringTableEntry {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "language")]
    pub language: String,
    #[serde(rename = "text")]
    pub text: String,
    #[serde(rename = "original")]
    pub original: String,
    #[serde(rename = "file")]
    pub file: String,
    #[serde(rename = "node")]
    pub node: String,
    #[serde(rename = "lineNumber")]
    pub line_number: String,
    #[serde(rename = "lock")]
    pub lock: String,
    #[serde(rename = "comment")]
    pub comment: String,
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct GDStringTableEntry {
    base: Base<Resource>,
    #[export]
    pub id: GString,
    #[export]
    pub language: GString,
    #[export]
    pub text: GString,
    #[export]
    pub original: GString,
    #[export]
    pub file: GString,
    #[export]
    pub node: GString,
    #[export]
    pub line_number: GString,
    #[export]
    pub lock: GString,
    #[export]
    pub comment: GString,
}

#[godot_api]
impl GDStringTableEntry {
    pub fn from_string_table_entry(string_table_entry: &StringTableEntry) -> Gd<Self> {
        return Gd::from_init_fn(|base| {
            return Self{
                base,
                id: string_table_entry.id.to_godot(),
                language: string_table_entry.language.to_godot(),
                text: string_table_entry.text.to_godot(),
                original: string_table_entry.original.to_godot(),
                file: string_table_entry.file.to_godot(),
                node: string_table_entry.node.to_godot(),
                line_number: string_table_entry.line_number.to_godot(),
                lock: string_table_entry.lock.to_godot(),
                comment: string_table_entry.comment.to_godot(),
            }
        })
    }

    pub fn to_string_table_entry(&self) -> StringTableEntry {
        return StringTableEntry {
            id: self.id.to_string(),
            language: self.language.to_string(),
            text: self.text.to_string(),
            original: self.original.to_string(),
            file: self.file.to_string(),
            node: self.node.to_string(),
            line_number: self.line_number.to_string(),
            lock: self.lock.to_string(),
            comment: self.comment.to_string(),
        }
    }

    #[func]
    pub fn parse_from_csv(source_text: GString) -> Array<Gd<GDStringTableEntry>> {
        let string_text = source_text.to_string();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .from_reader(string_text.as_bytes());

        let mut data: Array<Gd<GDStringTableEntry>> = array![];
        for result in reader.deserialize() {
            let entry: StringTableEntry = result.unwrap();
            data.push(GDStringTableEntry::from_string_table_entry(&entry));
        }
        return data;
    }

    #[func]
    pub fn create_csv(entries: Array<Gd<GDStringTableEntry>>) -> GString {
        let mut writer = csv::WriterBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .from_writer(vec![]);

        for entry in entries.iter_shared() {
            let record = entry.bind().to_string_table_entry();
            writer.serialize(record).expect("Failed to serialize record");
        }
        writer.flush().expect("Failed to flush CSV writer");

        return String::from_utf8(writer.into_inner().unwrap()).unwrap().to_godot();
    }
}

impl PartialEq<Self> for GDStringTableEntry {
    fn eq(&self, other: &Self) -> bool {
        return self.language == other.language &&
            self.id == other.id &&
            self.text == other.text &&
            self.file == other.file &&
            self.node == other.node &&
            self.line_number == other.line_number &&
            self.lock == other.lock &&
            self.comment == other.comment;
    }
}