class_name YarnEditorUtility extends Object

const TEMPLATE_FILE_PATH = "res://addons/yarnspinner-gdextension/editor/YarnScriptTemplate.txt"

static func create_yarn_script(script_path: String):
	print("Creating new yarn script at %s" % script_path)
	
	var template_content = "---\n===\n"
	var file_access = FileAccess.open(TEMPLATE_FILE_PATH, FileAccess.READ)
	if file_access == null:
		push_warning("Failed to find the Yarn script template file. Creating an empty file instead.")
	else:
		template_content = file_access.get_as_text()
		
	var script_name = script_path.get_file().trim_suffix(script_path.get_extension()).replace(" ", "_")
	template_content = template_content.replace("#SCRIPTNAME#", script_name)
	
	template_content = RegEx.create_from_string("\r\n?|\n").sub(template_content, "\n", true)
	
	var full_path = ProjectSettings.globalize_path(script_path)
	file_access = FileAccess.open(full_path, FileAccess.WRITE)
	file_access.store_string(template_content)
	
	print("Wrote new file  %s" % script_path)
	EditorInterface.get_resource_filesystem().scan_sources();
	
static func create_yarn_project(project_path: String):
	var json_project = YarnProject.new().to_json()
	var file_access = FileAccess.open(ProjectSettings.globalize_path(project_path), FileAccess.WRITE)
	file_access.store_string(json_project)
	
static func create_markup_palette(palette_path: String):
	var new_palette = MarkupPalette.new()
	var abs_path = ProjectSettings.globalize_path(palette_path)
	new_palette.resource_name = abs_path.get_file().trim_suffix(abs_path.get_extension())
	new_palette.resource_path = palette_path
	if ResourceSaver.save(new_palette, palette_path) != OK:
		push_error("Failed to save markup palette to %s" % palette_path)
