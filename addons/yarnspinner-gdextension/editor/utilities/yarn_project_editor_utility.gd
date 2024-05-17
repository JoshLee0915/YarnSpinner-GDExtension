class_name YarnProjectEditorUtility extends Object

const KEEP_IMPORT_TEXT = "[remap]\n\nimporter=\"keep\""
const PROJECT_UPDATE_TIMEOUT = 80 # ms 

static var tmp = ""

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
