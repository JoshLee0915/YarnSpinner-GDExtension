class_name MarkupPaletteDeleteTagButton extends Button

var tag_name: String
var palette: MarkupPalette


# Called when the node enters the scene tree for the first time.
func _ready():
	pressed.connect(_on_pressed)

func _on_pressed():
	if not is_instance_valid(palette):
		return
	palette.colour_markers.erase(tag_name)
	ResourceSaver.save(palette, palette.resource_path)
	palette.notify_property_list_changed()
