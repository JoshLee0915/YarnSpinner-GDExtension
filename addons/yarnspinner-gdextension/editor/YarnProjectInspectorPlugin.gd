class_name YarnProjectInspectorPlugin extends EditorInspectorPlugin

var _compile_errors_property_editor: YarnCompileErrorsPropertyEditor
var _parse_error_control: ScrollContainer
var _project: YarnProject

var _file_name_label_scene: PackedScene = ResourceLoader.load("res://addons/yarnspinner-gdextension/editor/ui/FilenameLabel.tscn")
var _error_text_label_scene: PackedScene = ResourceLoader.load("res://addons/yarnspinner-gdextension/editor/ui/ErrorTextLabel.tscn")
var _context_label_scene: PackedScene = ResourceLoader.load("res://addons/yarnspinner-gdextension/editor/ui/ContextLabel.tscn")

var _error_container: VBoxContainer
var _source_scripts_list_label: RichTextLabel
var _local_text_entry: LineEditWithSubmit
var _add_local_connected: bool
var _pending_csv_file_local_code: String
var _base_locale_input: LineEdit

func _can_handle(object):
	return object is YarnProject
	
func _parse_property(object, type, name, hint_type, hint_string, usage_flags, wide):
	var project = object as YarnProject
	if project == null:
		return false
		
	if _is_tres_yarn_project(project):
		return true
		
	_project = project
	var hidden_properties = [
		"last_import_had_implicit_string_ids",
		"last_import_had_any_strings",
		"is_successfully_parsed",
		"import_path",
		"json_project_path",
		"compiled_yarn_program_json",
		"list_of_functions",
	]
	
	if name in hidden_properties:
		return true
		
	match name:
		"project_errors":
			_compile_errors_property_editor = YarnCompileErrorsPropertyEditor.new()
			add_property_editor(name, _compile_errors_property_editor)
			
			_parse_error_control = ScrollContainer.new()
			_parse_error_control.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			_parse_error_control.size_flags_vertical = Control.SIZE_EXPAND_FILL
			
			var error_area_height = 40
			if len(_project.project_errors) > 0:
				error_area_height = 200
			
			_parse_error_control.custom_minimum_size = Vector2(0, error_area_height)
			
			_error_container = VBoxContainer.new()
			_parse_error_control.add_child(_error_container)
			_compile_errors_property_editor.on_errors_update.connect(_render_compilation_errors)
			_render_compilation_errors()
			add_custom_control(_parse_error_control)
			return true
		"declarations":
			var header = HBoxContainer.new()
			var story_var_label = Label.new()
			story_var_label.text = " Story Variables"
			story_var_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			story_var_label.size_flags_vertical = Control.SIZE_EXPAND_FILL
			header.add_child(story_var_label)
			
			var decl_count_label = Label.new()
			decl_count_label.text = "None" if _project.declarations.is_empty() else str(len(_project.declarations))
			decl_count_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			decl_count_label.size_flags_vertical = Control.SIZE_EXPAND_FILL
			header.add_child(decl_count_label)
			add_custom_control(header)
			
			if not _project.declarations.is_empty():
				var scroll_container = ScrollContainer.new()
				scroll_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
				scroll_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
				
				var vbox = VBoxContainer.new()
				vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
				vbox.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
				scroll_container.add_child(vbox)
				
				for decl in _project.declarations:
					var label_text = "%s (%s)\n" % [decl.name, decl.yarn_type]
					if decl.is_implicit:
						label_text += "Implicitly declared.\n"
					else:
						label_text += "Declared in %s\n" % decl.source_yarn_asset_path
					
					var default_value = "Not Set"
					match decl.yarn_type:
						"String":
							default_value = decl.default_value_string
						"Boolean":
							default_value = decl.default_value_bool
						"Number":
							default_value = decl.default_value_number
					
					label_text += "Default value: %s\n" % default_value
					var label = _file_name_label_scene.instantiate() as Label
					label.text = label_text
					vbox.add_child(label)
				
				scroll_container.custom_minimum_size = Vector2(0, 150)
				add_custom_control(scroll_container)
			return true
		
	return false
	
func _parse_begin(object):
	_project = object as YarnProject
	if _is_tres_yarn_project(_project):
		var warning_label = Label.new()
		warning_label.text = "⚠ This YarnProject may have been created via Create Resource > YarnProject.\nInstead, create projects via Tools > YarnSpinner > Create Yarn Project.\nThis will create a .yarnproject file that you can work with, rather than a .tres file.\nYou should delete this .tres file."
		warning_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		return
	_project.load_from_file(ProjectSettings.globalize_path(_project.json_project_path))
	
	var recompile_button = Button.new()
	recompile_button.text = "Re-compile Scripts in Project"
	recompile_button.pressed.connect(_on_recompile_click)
	add_custom_control(recompile_button)
	
	var add_tags_button = Button.new()
	add_tags_button.text = "Add Line Tags to Scripts"
	add_tags_button.pressed.connect(_on_add_tags_clicked)
	add_custom_control(add_tags_button)
	
	var script_patterns_grid = GridContainer.new()
	script_patterns_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	script_patterns_grid.columns = 2
	script_patterns_grid.tooltip_text = "YarnSpinner-Godot will search for all .yarn files that match the \n list of patterns in sourceFiles in %s. Each pattern will be used to search the file system for files with names that match specified patterns sepcified in the yarn project.\nThese patterns are relative to the location of this YarnProject\nA list of .yarn files found this way will be displayed here." % _project.json_project_path
	
	for pattern in _project.get_source_file_patterns():
		var pattern_label = Label.new()
		pattern_label.text = pattern
		# TODO: Find a better way to handle this
		pattern_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		pattern_label.custom_minimum_size = Vector2(500, 0)
		script_patterns_grid.add_child(pattern_label)
		
		var pattern_delete_button = SourcePatternDeleteButton.new()
		pattern_delete_button.text = "x"
		pattern_delete_button.project = _project
		pattern_delete_button.pattern = pattern
		script_patterns_grid.add_child(pattern_delete_button)
	
	var new_pattern_label = Label.new()
	new_pattern_label.text = "New Pattern"
	new_pattern_label.tooltip_text = "Add a pattern that will match .yarn scripts you want to include.\n These patterns are relative to the directory in which this .yarnproject file is saved."
	script_patterns_grid.add_child(new_pattern_label)
	
	var script_pattern_input = LineEdit.new()
	script_pattern_input.placeholder_text = "**/*.yarn"
	script_pattern_input.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	script_patterns_grid.add_child(script_pattern_input)
	
	var add_patterns_button = SourcePatternAddButton.new()
	add_patterns_button.text = "Add"
	add_patterns_button.script_pattern_input = script_pattern_input
	add_patterns_button.project = _project
	script_patterns_grid.add_child(add_patterns_button)
	add_custom_control(script_patterns_grid)
	
	var num_scripts_text = "None"
	var source_scripts = _project.get_source_files()
	if not source_scripts.is_empty():
		num_scripts_text = "%d .yarn scripts" % len(source_scripts)
		
	var matching_scripts_header = HBoxContainer.new()
	matching_scripts_header.size_flags_vertical = Control.SIZE_EXPAND_FILL
	matching_scripts_header.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	
	var matching_scripts_label = Label.new()
	matching_scripts_label.text = "Matching Scripts"
	matching_scripts_header.add_child(matching_scripts_label)
	
	var num_scripts_label = Label.new()
	num_scripts_label.text = num_scripts_text
	num_scripts_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	num_scripts_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	matching_scripts_header.add_child(num_scripts_label)
	add_custom_control(matching_scripts_header)
	
	var script_area_height = 40
	if not source_scripts.is_empty():
		script_area_height = 180
		
	_source_scripts_list_label = RichTextLabel.new()
	_source_scripts_list_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_source_scripts_list_label.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_source_scripts_list_label.custom_minimum_size = Vector2(0, script_area_height)
	render_source_scripts_list()
	add_custom_control(_source_scripts_list_label)
	
	var local_grid = GridContainer.new()
	local_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	local_grid.columns = 3
	
	var localization_csvs_label = Label.new()
	localization_csvs_label.text = "Localization CSVs"
	local_grid.add_child(localization_csvs_label)
	
	var local_add_button = Button.new()
	local_add_button.text = "Add"
	_local_text_entry = LineEditWithSubmit.new()
	_local_text_entry.placeholder_text = "locale code"
	_local_text_entry.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_local_text_entry.submit_button = local_add_button
	local_grid.add_child(_local_text_entry)
	
	_local_text_entry.submit_button.pressed.connect(_locale_added)
	local_grid.add_child(_local_text_entry.submit_button)
	
	local_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	local_grid.size_flags_vertical = Control.SIZE_EXPAND_FILL
	
	var localizations = _project.get_localization()
	for local_key in localizations:
		var local_label = Label.new()
		local_label.text = local_key
		local_grid.add_child(local_label)
		
		var picker = HBoxContainer.new()
		picker.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		picker.size_flags_vertical = Control.SIZE_EXPAND_FILL
		
		var path_label = Label.new()
		path_label.text = localizations[local_key].strings
		path_label.size_flags_vertical = Control.SIZE_EXPAND_FILL
		path_label.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY		
		if path_label.text.is_empty():
			path_label.text = "(none)"
		path_label.custom_minimum_size = Vector2(80, 30)
		path_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		path_label.clip_text = true
		picker.add_child(path_label)
		
		var picker_button = Button.new()
		picker_button.text = "Browse"
		_pending_csv_file_local_code = local_key
		picker_button.pressed.connect(_select_local_csv_path)
		picker.add_child(picker_button)
		local_grid.add_child(picker)
		
		var delete_button = LocaleDeleteButton.new()
		delete_button.text = "X"
		delete_button.local_code = local_key
		local_grid.add_child(delete_button)
	add_custom_control(local_grid)
	
	var base_local_row = HBoxContainer.new()
	base_local_row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	
	var base_lang_label = Label.new()
	base_lang_label.text = "Base language"
	base_local_row.add_child(base_lang_label)
	
	var change_base_local_button = Button.new()
	change_base_local_button.text = "Change"
	_base_locale_input = LineEditWithSubmit.new()
	_base_locale_input.text = _project.get_base_language()
	_base_locale_input.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_base_locale_input.submit_button = change_base_local_button
	base_local_row.add_child(_base_locale_input)
	
	change_base_local_button.pressed.connect(_on_base_local_changed)
	
	base_local_row.add_child(change_base_local_button)
	add_custom_control(base_local_row)
	
	var write_base_csv_button = Button.new()
	write_base_csv_button.text = "Generate Godot Translation CSV"
	write_base_csv_button.tooltip_text = "Write all of the line ids in your Yarn Project to a godot translation CSV."
	write_base_csv_button.pressed.connect(_on_base_language_csv_clicked)
	add_custom_control(write_base_csv_button)
	
func remove_local(locale_code):
	if not is_instance_valid(_project):
		return
		
	print("Removed locale code %s" % locale_code)
	_project.remove_localization(locale_code)
	_project.save_project()
	_project.notify_property_list_changed()

func csv_file_selected(save_path: String):
	save_path = ProjectSettings.localize_path(save_path)
	print("CSV file selected for locale %s: %s" % [_pending_csv_file_local_code, save_path])
	if not _project.localization_contains_key(_pending_csv_file_local_code):
		_project.set_localization(_pending_csv_file_local_code, YarnLocalizationInfo.new())
	
	var localization = _project.get_localization()[_pending_csv_file_local_code] as YarnLocalizationInfo
	localization.strings = save_path
	_project.set_localization(_pending_csv_file_local_code, localization)
	_project.save_project()
	_project.notify_property_list_changed()

func render_compilation_errors():
	if not is_instance_valid(_project):
		return
		
	var errors = _project.project_errors
	_set_errors(errors)
	_project.notify_property_list_changed()
	
func render_source_scripts_list():
	if not is_instance_valid(_project):
		return
		
	var scripts = _project.get_source_files()
	_set_source_scripts(scripts)
	_project.notify_property_list_changed()

func _on_recompile_click():
	if not is_instance_valid(_project):
		return
		
	YarnCompiler.compile_all_scripts(_project)
	YarnProjectEditorUtility.save_yarn_project(_project)
	_compile_errors_property_editor._refresh()
	_project.notify_property_list_changed()
	
func _on_add_tags_clicked():
	if not is_instance_valid(_project):
		return
		
	YarnProjectEditorUtility.add_line_tags_to_files_in_yarn_project(_project)
	_compile_errors_property_editor._refresh()
	_project.notify_property_list_changed()
	
func _is_tres_yarn_project(project: YarnProject):
	return project.json_project_path.is_empty() || project.resource_path.to_lower().ends_with(".tres")
	
func _render_compilation_errors():
	if not is_instance_valid(_project):
		return
	_set_errors(_project.project_errors)
	_project.notify_property_list_changed()
	
func _set_errors(errors: Array[YarnProjectError]):
	for child in _error_container.get_children():
		child.queue_free()
		
	var error_groups = {}
	for error in errors:
		if not error_groups.has(error.file_name):
			error_groups[error.file_name] = []
		error_groups[error.file_name].push_back(error)
	
	for file_name in error_groups:
		var errors_in_file = error_groups[file_name] as Array[YarnProjectError]
		var file_name_label = _file_name_label_scene.instantiate() as Label
		var res_file_name = ProjectSettings.localize_path(file_name)
		file_name_label.text = res_file_name
		_error_container.add_child(file_name_label)
		
		var separator = HSeparator.new()
		separator.custom_minimum_size = Vector2(0, 4)
		separator.size_flags_horizontal = Control.SIZE_EXPAND
		_error_container.add_child(separator)
		
		for error in errors_in_file:
			var error_text_label = _error_text_label_scene.instantiate() as Label
			error_text_label.text = "    %s" % error.message
			_error_container.add_child(error_text_label)
			
			var context_label = _context_label_scene.instantiate() as Label
			context_label.text = "    %s" % error.context
			_error_container.add_child(context_label)

func _set_source_scripts(scripts: Array[String]):
	_source_scripts_list_label.text = ""
	for script in scripts:
		var res_file_name = ProjectSettings.localize_path(script.replace("\\", "/"))
		_source_scripts_list_label.text += res_file_name + "\n"

func _locale_added():
	_project.set_localization(_local_text_entry.text, YarnLocalizationInfo.new())
	_project.save_project()
	EditorInterface.get_resource_filesystem().scan_sources()
	_project.notify_property_list_changed()

func _select_local_csv_path():
	var dialog = FileDialog.new()
	dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.title = "Select CSV Path for Locale %s" % _pending_csv_file_local_code
	dialog.add_filter("*.csv; CSV File")
	dialog.files_selected.connect(csv_file_selected)
	EditorInterface.get_base_control().add_child(dialog)
	dialog.popup_centered(Vector2i(800, 600))

func _on_base_local_changed():
	if not is_instance_valid(_project):
		return
	_project.set_base_language(_base_locale_input.text.strip_edges())
	_project.save_project()
	EditorInterface.get_resource_filesystem().scan_sources()

func _on_base_language_csv_clicked():
	var dialog = FileDialog.new()
	dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.title = "Select CSV Path for the translation file"
	dialog.add_filter("*.csv; CSV File")
	dialog.file_selected.connect(_on_base_language_csv_file_selected)
	EditorInterface.get_base_control().add_child(dialog)
	dialog.popup_centered(Vector2i(700, 500))

func _on_base_language_csv_file_selected(save_path: String):
	YarnProjectEditorUtility.write_base_language_strings_csv(_project, save_path)
