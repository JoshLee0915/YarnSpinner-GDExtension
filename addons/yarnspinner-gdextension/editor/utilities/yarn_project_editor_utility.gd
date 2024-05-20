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
	
static func update_localization_csvs(project: YarnProject):
	var localization_csvs = project.get_localization()
	if !localization_csvs.is_empty():
		var modified_files = []
		if project.base_localization == null:
			YarnCompiler.compile_all_scripts(project)
			
		var localization = project.get_localization()
		for key in localization:
			var localization_info = localization[key] as YarnLocalizationInfo
			if localization_info.strings.is_empty():
				push_error("Can't update localization for %s because it doesn't have a Strings file." % key)
				continue
				
			var file_was_changed = _update_localization_file(project.base_localization.get_string_table_entries(), key, localization_info.strings)
			if file_was_changed:
				modified_files.push_back(localization_info.strings)
				
		if !modified_files.is_empty():
			print("Updated the following files: %s" % ", ".join(modified_files))
		else:
			print("No Localization CSV files needed updating.")
			
static func _update_localization_file(base_localization_strings: Array[GDStringTableEntry], language: String, csv_resource_path: String, generate_translation = true):
	var abs_csv_path = ProjectSettings.globalize_path(csv_resource_path)
	
	# Tracks if the translated localisation needed modifications
	# (either new lines added, old lines removed, or changed lines
	# flagged)
	var modificaions_needed = false
	
	var translated_strings: Array[GDStringTableEntry] = []
	if FileAccess.file_exists(abs_csv_path):
		var existing_csv_text = FileAccess.get_file_as_string(abs_csv_path)
		translated_strings = GDStringTableEntry.parse_from_csv(existing_csv_text)
	else:
		print("CSV file %s did not exist for locale %s. A new file will be created at that location." % [csv_resource_path, language])
		modificaions_needed = true
		
	# Convert both enumerables to dictionaries, for easier lookup
	# The list of line IDs present in each localisation	
	var base_ids = []
	var base_dictionary = {}
	for entry in base_localization_strings:
		base_dictionary[entry.id] = entry
		base_ids.push_back(entry.id)
	
	var translated_ids = []
	var translated_dictionary = {}
	for entry in translated_strings:
		translated_dictionary[entry.id] = entry
		translated_ids.push_back(entry.id)
		if base_dictionary.has(entry.id):
			entry.original = base_dictionary[entry.id].text
	
	# The list of line IDs that are ONLY present in each localisation
	var only_in_base_ids = base_ids.filter(func(id): return not translated_ids.has(id))
	var only_in_translated_ids = translated_ids.filter(func(id): return not base_ids.has(id))
	
	# Remove every entry whose ID is only present in the translated set. 
	# This entry has been removed from the base localization.
	for id in only_in_base_ids:
		var base_entry = base_dictionary[id] as GDStringTableEntry
		base_entry.file = ProjectSettings.localize_path(base_entry.file)
		var new_entery = GDStringTableEntry.new()
		new_entery.comment = base_entry.comment
		new_entery.file = base_entry.file
		new_entery.id = base_entry.id
		new_entery.line_number = base_entry.line_number
		new_entery.lock = base_entry.lock
		new_entery.node = base_entry.node
		
		# Empty this text, so that it's apparent that a translated version needs to be provided.
		new_entery.text = ""
		new_entery.original = base_entry.text
		new_entery.language = language
		
		translated_dictionary[id] = new_entery
		modificaions_needed = true
	
	# Finally, we need to check for any entries in the translated localisation that:
	# 1. have the same line ID as one in the base, but
	# 2. have a different Lock (the hash of the text), which indicates that the base text has changed.
	# First, get the list of IDs that are in both base and translated, and then filter this list to 
	# any where the lock values differ
	var out_of_date_lock_ids = []
	for id in base_dictionary:
		if translated_dictionary.has(id) && base_dictionary[id].lock != translated_dictionary[id].lock:
			out_of_date_lock_ids.push_back(id)
			
	# Now loop over all of these, and update our translated dictionary to include a note that 
	# it needs attention
	for id in out_of_date_lock_ids:
		var entry = translated_dictionary[id]
		entry.text = "(NEEDS UPDATE) %s" % entry.text
		entry.original = base_dictionary[id].text
		entry.lock = base_dictionary[id].lock
		translated_dictionary[id] = entry
		modificaions_needed = true
		
	# We're all done!
	if not modificaions_needed:
		if generate_translation:
			_generate_godot_translation(language, csv_resource_path)
		return false
		
	# We need to produce a replacement CSV file for the translated entries.
	var output_string_entries = translated_dictionary.values()
	output_string_entries.sort_custom(
		func (a: GDStringTableEntry, b: GDStringTableEntry):
			return a.file < b.file || int(a.line_number) < int(b.line_number))
	var output_csv = GDStringTableEntry.create_csv(output_string_entries)
	
	# Write out the replacement text to this existing file, replacing 
	# its existing contents
	FileAccess.open(abs_csv_path, FileAccess.WRITE).store_string(output_csv)
	var csv_import = "%s.import" % abs_csv_path
	if not FileAccess.file_exists(csv_import):
		FileAccess.open(csv_import, FileAccess.WRITE).store_string("[remap]\n\nimporter=\"keep\"")
	if generate_translation:
		_generate_godot_translation(language, csv_resource_path)
		
	return true
	
static func _generate_godot_translation(language: String, csv_file_path: String):
	var abs_csv_path = ProjectSettings.globalize_path(csv_file_path)
	var translation = Translation.new()
	translation.locale = language
	
	var csv_text = FileAccess.get_file_as_string(abs_csv_path)
	var string_entries = GDStringTableEntry.parse_from_csv(csv_text)
	for entry in string_entries:
		translation.add_message(entry.id, entry.text)
	
	var extension_regex = RegEx.create_from_string(".csv$")
	var translation_path = extension_regex.sub(abs_csv_path, ".translation")
	var translation_res_path = ProjectSettings.localize_path(translation_path)
	ResourceSaver.save(translation, translation_res_path)
	print("Wrote translation file for %s to %s." % [language, translation_res_path])
	
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
	_update_localization_file(project.base_localization.get_string_table_entries(), project.get_base_language(), path, false)
