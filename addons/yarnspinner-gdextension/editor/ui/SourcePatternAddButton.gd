class_name SourcePatternAddButton extends Button

var script_pattern_input: LineEdit
var project: YarnProject

func _ready():
	pressed.connect(_on_pressed)

func _on_pressed():
	if not is_instance_valid(project) || not is_instance_valid(script_pattern_input):
		return
		
	if script_pattern_input.text.is_empty():
		return
		
	var exsiting_patterns = project.get_source_file_patterns()
	if exsiting_patterns.any(func(pattern): return pattern == script_pattern_input.text):
		print("Not adding duplicate pattern '%s'" % script_pattern_input.text)
		return
	
	project.add_source_file_pattern(script_pattern_input.text)
	project.save_project()
	EditorInterface.get_resource_filesystem().scan_sources()
	project.notify_property_list_changed()
