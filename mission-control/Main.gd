extends Spatial

onready var sensor = Sensor.new()

var connected = false
var throttle_value = 0.0
var led = false
var stab = false
var mode = 0

const mode_to_command = {
	0: {"All": {}},
	1: {"X1": {}},
	2: {"X2": {}},
	3: {"X3": {}},
	4: {"X4": {}},
}

func _process(_delta):
	var t = $Control/MissionControl/Throttle/Value
	t.value = throttle_value * t.max_value

	var s = $Control/MissionControl/Stabilisation
	s.pressed = stab

func _input(event):
	if InputMap.event_is_action(event, "throttle"):
		throttle_value = event.get_axis_value()
	if Input.is_action_pressed("ui_cancel"):
		led = !led
		send_command({"Led": led})
	if Input.is_action_pressed("stab"):
		stab = !stab
		send_command({"Stabilisation": stab})
	if Input.is_action_pressed("ui_select"):
		mode = (mode + 1) % 5
		send_mode(mode)

func _on_Button_button_up():
	var sensor_mac = $Control/MissionControl/Connection/MAC.text
	var response: Dictionary = sensor.connect(sensor_mac)
	if response.has("Ok"):
		connected = true
		$Control/MissionControl/Connect.icon = load("res://assets/connected.png")
	else:
		print(response.get("Err"))

func _on_Timer_timeout():
	send_command({"Throttle": throttle_value})

func _on_All_button_up():
	mode = 0
	send_mode(mode)

func _on_X1_button_up():
	mode = 1
	send_mode(mode)

func _on_X2_button_up():
	mode = 2
	send_mode(mode)

func _on_X3_button_up():
	mode = 3
	send_mode(mode)

func _on_X4_button_up():
	mode = 4
	send_mode(mode)

func send_mode(m: int):
	send_command({"SwitchMode": mode_to_command[m]})

func send_command(command: Dictionary):
	if connected:
		var result: Dictionary = sensor.send_command(command)
		if not result.has("Ok"):
			print(result.get("Err"))


func _on_TelemetryTimer_timeout():
	pass
	#if connected:
	#	print(sensor.get_angles())


func _on_GetAngles_button_up():
	if connected:
		print(sensor.get_angles())
