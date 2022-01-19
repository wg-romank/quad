extends Spatial


onready var sensor = Sensor.new()

var connected = false

var throttle_on = false
var throttle_value = 0.0

func _ready():
	$Control/GridContainer/HBoxContainer2/TextureRect.texture = load("res:/assets/disconnected.png")
	
func _process(delta):
	$Control/GridContainer/TextureProgress.value = throttle_value * $Control/GridContainer/TextureProgress.max_value

func _input(event):
	if InputMap.event_is_action(event, "throttle"):
		throttle_value = event.get_axis_value()
	throttle_on =  Input.is_action_pressed("throttle_on")

	if connected:
		var result = sensor.send_throttle(throttle_on, throttle_value)
		if not result.has("Ok"):
			print(result.get("Err"))

func _on_Button_button_up():
	var sensor_mac = $Control/GridContainer/HBoxContainer/TextEdit.text
	var response = sensor.connect(sensor_mac)
	if response.has("Ok"):
		connected = true
		$Control/GridContainer/HBoxContainer2/TextureRect.texture = load("res://assets/connected.png")
	else:
		print(response.get("Err"))
