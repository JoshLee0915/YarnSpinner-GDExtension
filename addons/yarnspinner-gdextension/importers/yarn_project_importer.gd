class_name YarnProjectImporter extends EditorImportPlugin

func _get_recognized_extensions():
	return ["yarnproject"]
	
func _get_importer_name():
	return "yarnproject"
	
func _get_visible_name():
	return "Yarn Project"
	
func _get_save_extension():
	return "tres"
	
func _get_resource_type():
	return "Resource"
	
func _get_preset_count():
	return 0
	
func _get_priority():
	return 1.0
	
func _get_import_order():
	return 0
	
func _get_import_options(path, preset_index):
	return []
	
func _import(source_file, save_path, options, platform_variants, gen_files):
	var project: YarnProject = null
	var full_save_path = "%s.%s" % [save_path, _get_save_extension()]
	
	project = ResourceLoader.load(source_file) as YarnProject
	if project == null:
		project = YarnProject.new()
	project.json_project_path = source_file
	project.import_path = full_save_path
	project.resource_name = source_file.get_file().trim_suffix(source_file.get_extension())
	
	var save_err = ResourceSaver.save(project, project.import_path)
	if save_err != OK:
		push_error("Error saving .yarnproject file import %s" % source_file)
		return save_err
	
	YarnProjectEditorUtility.update_yarn_project(project)
	return OK
