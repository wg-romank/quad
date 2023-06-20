extends Spatial

onready var sensor = Sensor.new()

var connected = false
var throttle_value = 0.0
var led = false
var stab = false
var motors = 0b1111

const X1 = 0b1000
const X2 = 0b0100
const X3 = 0b0010
const X4 = 0b0001

func _process(_delta):
	var t = $Control/MissionControl/Throttle/Value
	t.value = throttle_value * t.max_value

	var s = $Control/MissionControl/Stabilisation
	s.pressed = stab

	btn_enabled($Control/MissionControl/Mode/X1, motors & X1)
	btn_enabled($Control/MissionControl/Mode/X2, motors & X2)
	btn_enabled($Control/MissionControl/Mode/X3, motors & X3)
	btn_enabled($Control/MissionControl/Mode/X4, motors & X4)

func btn_enabled(btn: Button, value: int):
	if connected:
		if value != 0:
			btn.icon = load("res://assets/connected.png")
		else:
			btn.icon = load("res://assets/disconnected.png")

func _input(event):
	if InputMap.event_is_action(event, "throttle"):
		throttle_value = event.get_axis_value()
	if Input.is_action_pressed("ui_cancel"):
		led = !led
		send_command({"Led": led})
	if Input.is_action_pressed("stab"):
		stab = !stab
		send_command({"Stabilisation": stab})

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

func _on_X1_button_up():
	motors ^= X1
	send_mode(motors)

func _on_X2_button_up():
	motors ^= X2
	send_mode(motors)

func _on_X3_button_up():
	motors ^= X3
	send_mode(motors)

func _on_X4_button_up():
	motors ^= X4
	send_mode(motors)

func send_mode(m: int):
	send_command({"SwitchMode": m})

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
