class_name YarnMarkupPaletteInspectorPlugin extends EditorInspectorPlugin

func _can_handle(object):
	return object is MarkupPalette
	
func _parse_property(object, type, name, hint_type, hint_string, usage_flags, wide):
	var palette = object as MarkupPalette
	if palette == null:
		return false
		
	if name == "colour_markers":
		var label = Label.new()
		label.text = "Map [markup] tag names to colors"
		add_custom_control(label)
		
		if len(palette.colour_markers) == 0:
			var no_color_label = Label.new()
			no_color_label.text = "No colors remapped"
			add_custom_control(no_color_label)
		else:
			var color_remap_grid = GridContainer.new()
			color_remap_grid.columns = 3
			color_remap_grid.size_flags_vertical = Control.SIZE_EXPAND_FILL
			color_remap_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			
			var original_header = Label.new()
			original_header.text = "Markup Tag"
			color_remap_grid.add_child(original_header)
			
			var replacement_header = Label.new()
			replacement_header.text = "Text Color"
			color_remap_grid.add_child(replacement_header)
			
			var delete_header = Label.new()
			delete_header.text = "Delete"
			color_remap_grid.add_child(delete_header)
			
			const REMAP_HEIGHT = 4
			for tag_name in palette.colour_markers.keys():
				var tag_label = Label.new()
				tag_label.text = tag_name
				color_remap_grid.add_child(tag_label)
				
				var replacement_color_button = MarkupPaletteColorButton.new()
				replacement_color_button.palette = palette
				replacement_color_button.tag_name = tag_name
				replacement_color_button.color = palette.color_for_marker(tag_name)
				replacement_color_button.size = Vector2(0, REMAP_HEIGHT)
				replacement_color_button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
				color_remap_grid.add_child(replacement_color_button)
				
				var delete_area = HBoxContainer.new()
				var delete_spacer = Label.new()
				delete_spacer.text = "   "
				
				var delete_button = MarkupPaletteDeleteTagButton.new()
				delete_button.text = "x"
				delete_button.tag_name = tag_name
				delete_button.palette = palette
				delete_button.add_theme_color_override("normal", Color.RED)
				delete_button.size = Vector2(4, REMAP_HEIGHT)
				delete_button.size_flags_horizontal = 0
				
				delete_area.add_child(delete_spacer)
				delete_area.add_child(delete_button)
				color_remap_grid.add_child(delete_area)
			add_custom_control(color_remap_grid)

		var new_tag_row = HBoxContainer.new()
		var add_new_tag_button = MarkupPaletteAddTagButton.new()
		add_new_tag_button.text = "Add"
		add_new_tag_button.palette = palette
		
		var new_tag_name_input = LineEditWithSubmit.new()
		new_tag_name_input.placeholder_text = "tag name, without []"
		new_tag_name_input.custom_minimum_size = Vector2(80, 10)
		new_tag_name_input.submit_button = add_new_tag_button
		
		add_new_tag_button.new_tag_name_input = new_tag_name_input
		new_tag_name_input.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		add_new_tag_button.disabled = true
		
		new_tag_row.add_child(new_tag_name_input)
		new_tag_row.add_child(add_new_tag_button)
		new_tag_row.size_flags_vertical = Control.SIZE_EXPAND_FILL
		new_tag_row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		add_custom_control(new_tag_row)
		
		return true

	return false
