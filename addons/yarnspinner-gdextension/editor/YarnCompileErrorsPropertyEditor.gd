class_name YarnCompileErrorsPropertyEditor extends EditorProperty

signal on_errors_update(yarn_project: Object)

var _property_control = Label.new()
var _current_value = []

func _init():
	label = "Project Errors"
	add_child(_property_control)
	add_focusable(_property_control)
	_refresh_control_text()

func _update_property():
	var new_variant_value = get_edited_object().get(get_edited_property())
	var new_value = new_variant_value as Array
	if new_value == _current_value:
		return
		
	_current_value = new_value
	_refresh_control_text()
	on_errors_update.emit()
	
func _refresh_control_text():
	if _current_value.is_empty():
		_property_control.text = "None"
	else:
		_property_control.text = "❌%s error(s)" % _current_value.size()
		
func _refresh():
	emit_changed(get_edited_property(), get_edited_object().get(get_edited_property()))
