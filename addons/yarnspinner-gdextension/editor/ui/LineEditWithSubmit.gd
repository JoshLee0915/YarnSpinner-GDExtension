class_name LineEditWithSubmit extends LineEdit

var submit_button: Button

func _ready():
	text_changed.connect(_on_text_changed)

func _on_text_changed(new_text: String):
	if not is_instance_valid(submit_button):
		return
	submit_button.disabled = new_text.is_empty()
