extends YarnVariableStorage

@export var store: Dictionary = {}

func _contains_impl(variable_name: String) -> bool:
	return variable_name in store.keys()
	
func _get_variables_impl() -> Dictionary:
	return store
	
func _get_variable_impl(variable_name: String) -> Variant:
	return store[variable_name]
	
func _set_variable_impl(variable_name: String, value: Variant) -> void:
	store[variable_name] = value
	
func _clear_impl() -> void:
	store.clear()
