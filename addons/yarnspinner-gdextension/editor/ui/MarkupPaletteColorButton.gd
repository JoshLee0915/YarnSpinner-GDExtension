class_name MarkupPaletteColorButton extends ColorPickerButton

var tag_name: String
var palette: MarkupPalette

# Called when the node enters the scene tree for the first time.
func _ready():
	popup_closed.connect(_on_popup_closed)

func _on_popup_closed():
	if not is_instance_valid(palette):
		return
	palette.colour_markers[tag_name] = color
	ResourceSaver.save(palette, palette.resource_path)
	palette.notify_property_list_changed()
