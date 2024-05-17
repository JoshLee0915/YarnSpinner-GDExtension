@tool
extends EditorPlugin

class ToolsMenuItem:
	var handler: Callable
	var menu_name: String
	
	func _init(menu_name: String, handler: Callable):
		self.menu_name = menu_name
		self.handler = handler
		
const TOOLS_MENU_NAME = "YarnSpinner"

var _import_plugins: Array[EditorImportPlugin] = [YarnImporter.new(), YarnProjectImporter.new()]
var _inspector_plugins: Array[EditorInspectorPlugin] = [YarnMarkupPaletteInspectorPlugin.new()]
var _tools_menu_items: Dictionary = {
	0: ToolsMenuItem.new("Create Yarn Script", _create_yarn_script),
	1: ToolsMenuItem.new("Create Yarn Project", _create_yarn_project),
	2: ToolsMenuItem.new("Create Markup Palette", _create_markup_palette),
}

var _popup = PopupMenu.new()

func _enter_tree():
	for plugin in _import_plugins:
		add_import_plugin(plugin)
		
	for plugin in _inspector_plugins:
		add_inspector_plugin(plugin)
		
	for key in _tools_menu_items:
		_popup.add_item(_tools_menu_items[key].menu_name, key)
	_popup.id_pressed.connect(self._on_popup_id_pressed)
	add_tool_submenu_item(TOOLS_MENU_NAME, _popup)
	
	# TODO: Add types


func _exit_tree():
	for plugin in _import_plugins:
		if is_instance_valid(plugin):
			remove_import_plugin(plugin)
			
	# TODO: Remove Types
			
	for plugin in _inspector_plugins:
		if is_instance_valid(plugin):
			remove_inspector_plugin(plugin)
	
	remove_tool_menu_item(TOOLS_MENU_NAME)
			
func _on_popup_id_pressed(id: int):
	var menu_item = _tools_menu_items.get(id) as ToolsMenuItem
	if menu_item != null:
		menu_item.handler.call()
			
func _create_yarn_script():
	print("Opening 'create yarn script' menu")
	_show_create_file_popup("*.yarn ; Yarn Script", "Create a new Yarn Script", _create_yarn_script_destination_selected)
	
func _create_yarn_script_destination_selected(dest: String):
	print("Creating a yarn script at %s" % dest)
	YarnEditorUtility.create_yarn_script(dest)
	
func _create_yarn_project():
	print("Opening 'create yarn project' menu")
	_show_create_file_popup("*.yarnproject; Yarn Project", "Create a new Yarn Project", _create_yarn_project_destination_selected)
	
func _create_yarn_project_destination_selected(dest: String):
	print("Creating a yarn project at %s" % dest)
	YarnEditorUtility.create_yarn_project(dest)
	EditorInterface.get_resource_filesystem().scan_sources()
	
func _create_markup_palette():
	print("Opening 'create markup palette' menu")
	_show_create_file_popup("*.tres; Markup Palette", "Create a new Markup Palette", _create_markup_palette_destination_selected)
	
func _create_markup_palette_destination_selected(dest: String):
	print("Creating a markup palette at %s" % dest)
	YarnEditorUtility.create_markup_palette(dest)

func _show_create_file_popup(filter: String, window_title: String, file_select_handler: Callable):		
	var dialog = EditorFileDialog.new()
	dialog.add_filter(filter)
	dialog.file_mode = EditorFileDialog.FILE_MODE_SAVE_FILE
	dialog.title = window_title
	dialog.file_selected.connect(file_select_handler)
	EditorInterface.get_base_control().add_child(dialog)
	dialog.popup_centered(Vector2i(700, 500))
