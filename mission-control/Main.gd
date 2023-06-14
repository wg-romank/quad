extends Spatial


onready var sensor = Sensor.new()

var connected = false

var throttle_value = 0.0
var led = false
var stab = false
var mode = 0

func _process(_delta):
	var t = $Control/MissionControl/Throttle/Value
	t.value = throttle_value * t.max_value

	var s = $Control/MissionControl/Stabilisation
	s.pressed = stab

func _input(event):
	var result
	if InputMap.event_is_action(event, "throttle"):
		throttle_value = event.get_axis_value()
	if Input.is_action_pressed("ui_cancel"):
		led = !led
		result = sensor.led(led)
		if not result.has("Ok"):
			print(result.get("Err"))
	if Input.is_action_pressed("stab"):
		stab = !stab
		result = sensor.stab(stab)
		if not result.has("Ok"):
			print(result.get("Err"))
	if Input.is_action_pressed("ui_select"):
		mode = (mode + 1) % 5
		result = sensor.mode(mode)
		if not result.has("Ok"):
			print(result.get("Err"))

func _on_Button_button_up():
	var sensor_mac = $Control/MissionControl/Connection/MAC.text
	var response = sensor.connect(sensor_mac)
	if response.has("Ok"):
		connected = true
		$Control/MissionControl/Connect.icon = load("res://assets/connected.png")
	else:
		print(response.get("Err"))


func _on_Timer_timeout():
	if connected:
		var result = sensor.send_throttle(throttle_value)
		if not result.has("Ok"):
			print(result.get("Err"))


func _on_All_button_up():
	mode = 0
	if connected:
		var result = sensor.mode(mode)
		if not result.has("Ok"):
			print(result.get("Err"))


func _on_X1_button_up():
	mode = 1
	if connected:
		var result = sensor.mode(mode)
		if not result.has("Ok"):
			print(result.get("Err"))


func _on_X2_button_up():
	mode = 2
	if connected:
		var result = sensor.mode(mode)
		if not result.has("Ok"):
			print(result.get("Err"))


func _on_X3_button_up():
	mode = 3
	if connected:
		var result = sensor.mode(mode)
		if not result.has("Ok"):
			print(result.get("Err"))


func _on_X4_button_up():
	mode = 4
	if connected:
		var result = sensor.mode(mode)
		if not result.has("Ok"):
			print(result.get("Err"))
