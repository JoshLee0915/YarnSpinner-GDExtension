class_name YarnImporter extends EditorImportPlugin

func _get_recognized_extensions():
	return ["yarn"]
	
func _get_importer_name():
	return "yarnscript"
	
func _get_visible_name():
	return "Yarn Script"
	
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
	
func _get_import_options(_path, _preset_index):
	return {}
	
func _import(source_file, save_path, options, platform_variants, gen_files):
	var extension = source_file.get_extension()
	
	if extension == ".yarn":
		_import_yarn(source_file)
		
	var imported_marker_resource = Resource.new()
	imported_marker_resource.resource_name = ProjectSettings.globalize_path(source_file).get_file().trim_suffix(extension)
	
	var result = ResourceSaver.save(imported_marker_resource, "%s.%s" % [save_path, _get_save_extension()])
	if result != OK:
		push_error("Error saving yarn file import: %s" % result)
		
	return result
	
func _import_yarn(source_file: String):
	print("Importing Yarn script %s" % source_file)
	var project_path = YarnProjectEditorUtility.get_destination_project_path(source_file)
	if project_path == null:
		print("The yarn file {assetPath} is not currently associated with a Yarn Project." +
			" Create a Yarn Project by selecting YarnProject from the create new resource menu and make sure this" +
			" script matches one of the patterns defined for yarn source files.")
	else:
		var project = ResourceLoader.load(project_path)
		YarnProjectEditorUtility.update_yarn_project(project)
