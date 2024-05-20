use godot::engine::{ProjectSettings, ResourceSaver};
use godot::engine::global::Error;
use godot::engine::resource_saver::SaverFlags;
use godot::engine::utilities::push_error;
use godot::prelude::*;
use yarnspinner::compiler::Compiler;
use yarnspinner::core::Library;
use yarnspinner::core::Type::Function;
use yarnspinner::prelude::*;

use crate::gd_compilation::GDCompilation;
use crate::gd_declaration::GDDeclaration;
use crate::project::{YarnProject, YarnProjectError};

pub const YARN_COMPILER_SINGLETON_NAME: &str = "YarnCompiler";

#[derive(GodotClass)]
#[class(tool, init, base=Object)]
pub struct YarnCompilerSingleton {
    base: Base<Object>,
}

#[godot_api]
impl YarnCompilerSingleton {
    #[func]
    pub fn compile_all_scripts(mut project: Gd<YarnProject>) {
        let mut yarn_project = project.bind_mut();

        let path = yarn_project.base().get_path().to_string();
        let asset_path = std::path::Path::new(&path);

        yarn_project.base_mut().set_name(asset_path.file_name().unwrap().to_str().unwrap().to_godot());
        let source_scripts = yarn_project.get_source_files();
        if source_scripts.is_empty() {
            push_error(GString::from(format!("No .yarn files found matching the set pattern {}", yarn_project.get_source_files())).to_variant(), &[]);
            return;
        }

        yarn_project.project_errors = array![];

        // We can now compile!
        let mut yarn_files = vec![];
        for source_script in source_scripts.iter_shared() {
            if !source_script.is_empty() {
                let global_path = ProjectSettings::singleton().globalize_path(source_script).to_string();
                yarn_files.push(global_path.clone());
            }
        }

        if !yarn_files.is_empty() {
            match Self::compile_files(&yarn_files) {
                Ok(result) => {
                    if result.program.is_none() {
                        push_error("public error: Failed to compile: resulting program was null, but compiler did not report errors.".to_variant(), &[]);
                        return;
                    }

                    // Store _all_ declarations - both the ones in this
                    // .yarnproject file, and the ones inside the .yarn files.

                    // While we're here, filter out any declarations that begin with our
                    // Yarn public prefix. These are synthesized variables that are
                    // generated as a result of the compilation, and are not declared by
                    // the user.

                    let mut new_declarations = Array::<Gd<GDDeclaration>>::new();
                    for declaration in result.declarations {
                        if  declaration.name.starts_with("$Yarn.Internal.") {
                            continue;
                        }

                        if let Function(_) = declaration.r#type {
                            continue;
                        }

                        // try to re-use a declaration if one exists to avoid changing the .tres file so much
                        let mut existing_declaration = None;
                        for existing in yarn_project.declarations.iter_shared() {
                            if declaration.name == existing.bind().name.to_string() {
                                existing_declaration = Some(existing);
                                break;
                            }
                        }

                        match existing_declaration {
                            None => {
                                match GDDeclaration::from_declaration(&declaration) {
                                    Ok(decl) => new_declarations.push(decl),
                                    Err(e) => {
                                        panic!("{}", e)
                                    }
                                }
                            }
                            Some(decl) => {
                                new_declarations.push(decl);
                            }
                        }
                    }
                    yarn_project.declarations = new_declarations;
                    yarn_project.project_errors.clear();

                    yarn_project.list_of_functions = array![];
                    yarn_project.compiled_yarn_program_json = serde_json::to_string(&result.program.unwrap())
                        .expect("Unable to serialize Yarn Program to JSON")
                        .to_godot();

                    let path = yarn_project.import_path.clone();
                    drop(yarn_project);
                    let err = ResourceSaver::singleton()
                        .save_ex(project.upcast())
                        .path(path)
                        .flags(SaverFlags::REPLACE_SUBRESOURCE_PATHS)
                        .done();

                    if err != Error::OK {
                        push_error(format!("Failed to save updated YarnProject: {}", err.to_variant()).to_variant(), &[]);
                    }
                }
                Err(errors) => {
                    for info in errors.0 {
                        yarn_project.project_errors.push(YarnProjectError::new_gd());
                        let mut p = yarn_project.project_errors.last().expect("Failed to unwrap the newly created error");
                        let mut gd_error_mut = p.bind_mut();
                        gd_error_mut.file_name = ProjectSettings::singleton().localize_path(info.file_name.unwrap_or("".to_string()).to_godot());
                        gd_error_mut.message = info.message.to_godot();
                        gd_error_mut.context = info.context.unwrap_or("".to_string()).to_godot();

                        push_error(format!("Error compiling: {}", info.message).to_variant(), &[]);
                    }
                }
            }
        }
    }

    #[func]
    pub fn compile_yarn_files(yarn_files: Array<GString>) -> Gd<GDCompilation> {
        let mut yarn_file_paths = vec![];
        for source_script in yarn_files.iter_shared() {
            if !source_script.is_empty() {
                let global_path = ProjectSettings::singleton().globalize_path(source_script).to_string();
                yarn_file_paths.push(global_path.clone());
            }
        }

        return match Self::compile_files(&yarn_file_paths) {
            Ok(compilation) => GDCompilation::from_compilation(compilation),
            Err(errors) => GDCompilation::from_compilation_error(errors),
        }
    }

    #[func]
    pub fn add_tags_to_lines(contents: GString, existing_line_tags: Array<GString>) -> GString {
        return match Compiler::add_tags_to_lines(contents, existing_line_tags.iter_shared().map(|tag| { LineId::from(tag)}).collect()) {
            Ok(result) => result.unwrap_or("".to_string()).to_godot(),
            Err(err) => {
                for err_info in err.0 {
                    push_error(format!("Error: {}\nFile: {}", err_info.message, err_info.file_name.unwrap_or("None".to_string())).to_variant(), &[]);
                }
                return GString::from("");
            }
        }
    }

    fn compile_files(yarn_files: &Vec<String>) -> yarnspinner::compiler::Result<Compilation> {
        let mut job = YarnCompiler::new();
        job.compilation_type = CompilationType::FullCompilation;
        job.library = Library::standard_library();

        for path in yarn_files {
            job.read_file(&path);
        }

        return job.compile();
    }
}