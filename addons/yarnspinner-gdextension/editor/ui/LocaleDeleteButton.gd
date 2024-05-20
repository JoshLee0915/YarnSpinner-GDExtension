class_name LocaleDeleteButton extends Button

var local_code: String
var plugin: YarnProjectInspectorPlugin

func _ready():
	pressed.connect(on_pressed)

func on_pressed():
	if is_instance_valid(plugin):
		plugin.remove_local(local_code)

