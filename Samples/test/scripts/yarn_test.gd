extends Control

@export var dialogue_runner: DialogueRunner

@onready var label: RichTextLabel = $Panel/RichTextLabel


# Called when the node enters the scene tree for the first time.
func _ready():
	dialogue_runner.start_dialogue("HelloWorld")


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	dialogue_runner.continue_dialogue()


func _on_dialogue_runner_next_line(line: YarnLine):
	print(line.get_raw_text())
