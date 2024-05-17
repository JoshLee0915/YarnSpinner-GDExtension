class_name MarkupPaletteAddTagButton extends Button

var new_tag_name_input: LineEdit
var palette: MarkupPalette

# Called when the node enters the scene tree for the first time.
func _ready():
	pressed.connect(_on_pressed)

func _on_pressed():
	if not is_instance_valid(palette) or not is_instance_valid(new_tag_name_input):
		return
		
	var new_tag_name = new_tag_name_input.text.replace("[", "").replace("]", "")
	if new_tag_name.is_empty():
		print("Enter a markup tag name in order to add a color mapping.")
		return
	palette.colour_markers[new_tag_name] = Color.BLACK
	palette.notify_property_list_changed()
