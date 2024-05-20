use godot::builtin::{Array, array, dict, Dictionary, GString};
use godot::engine::Object;
use godot::obj::{Base, Gd};
use godot::prelude::{Export, GodotClass, GodotConvert, ToGodot, Var};
use yarnspinner::compiler::{Compilation, CompilerError, Diagnostic, DiagnosticSeverity};
use crate::gd_declaration::GDDeclaration;
use crate::gd_string_info::GDStringInfo;

#[derive(GodotConvert, Var, Export, Default)]
#[godot(via = GString)]
pub enum GDDiagnosticSeverity {
    #[default]
    Error,
    Warning,
}

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct GDDiagnosticRange {
    base: Base<Object>,
    #[var]
    pub start_line: u32,
    #[var]
    pub start_character: u32,
    #[var]
    pub end_line: u32,
    #[var]
    pub end_character: u32,
}

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct GDDiagnostic {
    base: Base<Object>,
    #[var]
    pub file_name: GString,
    #[var]
    pub range: Option<Gd<GDDiagnosticRange>>,
    #[var]
    pub message: GString,
    #[var]
    pub context: GString,
    #[var]
    pub severity: GDDiagnosticSeverity,
    #[var]
    pub start_line: u32
}

impl GDDiagnostic {
    pub fn from_diagnostic(diagnostic: &Diagnostic) -> Gd<Self> {
        let dg = diagnostic.clone();

        let range = match dg.range {
            None => None,
            Some(rng) => {
                Some(Gd::from_init_fn(|base1| {
                    return GDDiagnosticRange{
                        base: base1,
                        start_line: u32::try_from(rng.start.line).unwrap(),
                        start_character: u32::try_from(rng.start.character).unwrap(),
                        end_line: u32::try_from(rng.end.line).unwrap(),
                        end_character: u32::try_from(rng.end.character).unwrap(),
                    }
                }))
            }
        };

        let severity = match &dg.severity {
            DiagnosticSeverity::Error => GDDiagnosticSeverity::Error,
            DiagnosticSeverity::Warning => GDDiagnosticSeverity::Error,
        };

        return Gd::from_init_fn(|base| {
            return Self{
                base,
                file_name: dg.file_name.unwrap_or("".to_string()).to_godot(),
                range,
                message: dg.message.to_godot(),
                context: dg.context.unwrap_or("".to_string()).to_godot(),
                severity,
                start_line: u32::try_from(diagnostic.start_line).unwrap(),
            }
        })
    }
}

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct GDCompilation {
    base: Base<Object>,
    #[var]
    pub compiled_yarn_program_json: GString,
    #[var]
    pub string_table: Dictionary,
    #[var]
    pub declarations: Array<Gd<GDDeclaration>>,
    #[var]
    pub contains_implicit_string_tags: bool,
    #[var]
    pub file_tags: Dictionary,
    #[var]
    pub errors: Array<Gd<GDDiagnostic>>,
    #[var]
    pub warnings: Array<Gd<GDDiagnostic>>,
    #[var]
    pub debug_info: Dictionary
}

impl GDCompilation {
    pub fn from_compilation(compilation: Compilation) -> Gd<Self> {
        let compiled_yarn_program_json = serde_json::to_string(&compilation.program.unwrap())
            .expect("Unable to serialize Yarn Program to JSON")
            .to_godot();

        let mut string_table = dict!{};
        for (line_id, string_info) in compilation.string_table {
            let k = line_id.0;
            string_table.set(k.clone(), GDStringInfo::from_string_info(&string_info));
        }

        let mut declarations = array![];
        for declaration in compilation.declarations {
            declarations.push(GDDeclaration::from_declaration(&declaration).unwrap());
        }

        let mut warnings = array![];
        for wrn in compilation.warnings {
            warnings.push(GDDiagnostic::from_diagnostic(&wrn));
        }

        return Gd::from_init_fn(|base| {
            return Self {
                base,
                compiled_yarn_program_json,
                string_table,
                declarations,
                contains_implicit_string_tags: compilation.contains_implicit_string_tags,
                file_tags: Default::default(),
                errors: Default::default(),
                warnings,
                debug_info: Default::default(),
            }
        })
    }

    pub fn from_compilation_error(error: CompilerError) -> Gd<Self> {
        let mut errors = array![];
        for e in error.0 {
            errors.push(GDDiagnostic::from_diagnostic(&e));
        }

        return Gd::from_init_fn(|base| {
            return Self{
                base,
                compiled_yarn_program_json: Default::default(),
                string_table: Default::default(),
                declarations: Default::default(),
                contains_implicit_string_tags: false,
                file_tags: Default::default(),
                errors,
                warnings: Default::default(),
                debug_info: Default::default(),
            }
        })
    }
}
