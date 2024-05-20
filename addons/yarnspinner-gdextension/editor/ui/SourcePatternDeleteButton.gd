class_name SourcePatternDeleteButton extends Button

var pattern: String
var project: YarnProject

func _ready():
	pressed.connect(_on_pressed)

func _on_pressed():
	if not is_instance_valid(project):
		return
	project.remove_source_file(pattern)
	project.save_project()
	EditorInterface.get_resource_filesystem().scan_sources()
	project.notify_property_list_changed()
