class_name YarnProjectEditorUtility extends Object

const KEEP_IMPORT_TEXT = "[remap]\n\nimporter=\"keep\""
const PROJECT_UPDATE_TIMEOUT = 80 # ms 

static var tmp = ""

static func add_line_tags_to_files_in_yarn_project(project: YarnProject):
	# // First, gather all existing line tags across ALL yarn
	# projects, so that we don't accidentally overwrite an
	# existing one. Do this by finding all yarn scripts in all
	# yarn projects, and get the string tags inside them.
	
	var all_yarn_files = []
	for yp in load_all_yarn_projects() as Array[YarnProject]:
		for file in yp.get_source_files():
			if file != null:
				all_yarn_files.push_back(file)
				
	var all_existing_tags = []
	for path in all_yarn_files:
		var results = YarnCompiler.compile_yarn_files(path)
		if !results.errors.is_empty():
			push_error("Can't check for existing line tags in %s because it contains errors." % path)
			continue
		for tag_id in results.string_table:
			if !results.string_table[tag_id].is_implicit_tag:
				all_existing_tags.push_back(tag_id)
				
	var modified_files = []
	for script_file in project.get_source_files():
		var asset_path = ProjectSettings.globalize_path(script_file)
		var contents = FileAccess.open(asset_path, FileAccess.READ).get_as_text()
		
		var tagged_version = YarnCompiler.add_tags_to_lines(contents, all_existing_tags)
		if tagged_version.is_empty():
			continue
			
		if contents != tagged_version:
			modified_files.push_back(script_file)
			FileAccess.open(asset_path, FileAccess.WRITE).store_string(contents)
			
	if !modified_files.is_empty():
		print("Updated the following files: %s" % ", ".join(modified_files))
		EditorInterface.get_resource_filesystem().scan_sources()
	else:
		print("No files needed updating.")

static func get_destination_project_path(script_path: String):
	var dest_project_path = null
	var global_script_path = ProjectSettings.globalize_path(script_path)
	for project_local_path in _find_all_yarn_projects():
		var project_path = ProjectSettings.globalize_path(project_local_path).replace("\\", "/")
		var loaded_project = YarnProject.new()
		loaded_project.load_from_file(project_path)
		if loaded_project.get_source_files().any(global_script_path):
			dest_project_path = ProjectSettings.localize_path(project_path)
			break
	return dest_project_path
	
static func update_yarn_project(project: YarnProject):
	if project == null || project.resource_path.is_empty():
		return
	_update_yarn_project_task(project)
	
static func save_yarn_project(project: YarnProject):
	# TODO: Might need to reset some properties
	if project.json_project_path.is_empty():
		project.json_project_path = project.get_default_json_project_path()
		
	var save_result = ResourceSaver.save(project, project.import_path)
	if save_result != OK:
		push_error("Error updating YarnProject %s to %s: %s" % [project.resource_name, project.resource_path, save_result])
	else:
		print("Wrote updated YarnProject %s to %s" % [project.resource_name, project.resource_path])
		
static func load_all_yarn_projects():
	var projects = []
	for project_path in _find_all_yarn_projects():
		projects.push_back(ResourceLoader.load(project_path) as YarnProject)
	return projects
			
static func _update_localization_file(localizations: Array[Localization], csv_resource_path: String):
	var abs_csv_path = ProjectSettings.globalize_path(csv_resource_path)
	
	# Attempt to load any exsiting localization CSVs
	var exsiting_localizations = {}
	if FileAccess.file_exists(abs_csv_path):
		var existing_csv_text = FileAccess.get_file_as_string(abs_csv_path)
		for localization in Localization.parse_from_csv(existing_csv_text):
			exsiting_localizations[localization.local_code] = localization
	else:
		print("CSV file %s does not exist. A new file will be created at that location." % [csv_resource_path])
		
	# Merge with the passes localization files
	for localization in localizations:
		if not exsiting_localizations.has(localization.local_code):
			exsiting_localizations[localization.local_code] = localization
		else:
			for key in localization.string_table:
				exsiting_localizations[localization.local_code].add_localized_string_to_asset(key, localization.string_table[key])
				
	# Convert back to a csv
	var output_csv = Localization.generate_localization_csv(exsiting_localizations.values())
	FileAccess.open(abs_csv_path, FileAccess.WRITE).store_string(output_csv)
	EditorInterface.get_resource_filesystem().scan_sources()
	return true
	
static func _update_yarn_project_task(project: YarnProject):
	# Attempt to update the project file incase there where any changes
	project.load_from_file(project.json_project_path)
	YarnCompiler.compile_all_scripts(project)
	save_yarn_project(project)
	
static func _find_all_yarn_projects():
	var root_dir = ProjectSettings.globalize_path("res://")
	var project_files = []
	for project in _find_all_files(root_dir, ["**/*.yarnproject"]):
		project_files.append(ProjectSettings.localize_path(project))
	return project_files

static func _find_all_files(dir: String, patterns: Array[String]):
	var dir_access = DirAccess.open(dir)
	dir_access.list_dir_begin()
	
	var files = []
	var item = dir_access.get_next()
	while not item.is_empty():
		if dir_access.current_is_dir():
			files.append_array(_find_all_files(item, patterns))
		else:
			for pattern in patterns:
				if item.match(pattern):
					files.append(item)
					break
		item = dir_access.get_next()
	return files
	
static func write_base_language_strings_csv(project: YarnProject, path: String):
	_update_localization_file([project.base_localization], path)
