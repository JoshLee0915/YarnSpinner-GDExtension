extends Control

@export var dialogue_runner: DialogueRunner
@export var variable_store: YarnVariableStorage

@onready var label: RichTextLabel = $Panel/RichTextLabel
@onready var options_container: VBoxContainer = $CenterContainer/VBoxContainer

# Called when the node enters the scene tree for the first time.
func _ready():
	dialogue_runner.start_dialogue("HelloWorld")
	options_container.hide()
	
	variable_store.set_variable("$test", 1)
	if variable_store.contains("$test"):
		print("Stuff")


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	if Input.is_action_just_pressed("continue_dialogue") && !dialogue_runner.is_waiting_for_option_selection():
		dialogue_runner.continue_dialogue()


func _on_dialogue_runner_next_line(line: YarnLine):
	label.text = line.get_raw_text()

func _on_dialogue_runner_options_available(options: Array[YarnDialogueOption]) -> void:		
	for option in options:
		var btn = Button.new()
		btn.text = option.get_line().get_raw_text()
		btn.visible = option.is_available()
		btn.pressed.connect(func(): if dialogue_runner.is_waiting_for_option_selection(): dialogue_runner.select_option(option))
		options_container.add_child(btn)
	options_container.show()


func _on_dialogue_runner_option_selected(_option: YarnDialogueOption) -> void:
	options_container.hide()
	for child in options_container.get_children():
		child.queue_free()
