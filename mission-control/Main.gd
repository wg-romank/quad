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

func _input(event):
	if InputMap.event_is_action(event, "throttle"):
		throttle_value = event.get_axis_value()
	
	if connected:
		var result
		
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
			
		result = sensor.send_throttle(throttle_value)
		if not result.has("Ok"):
			print(result.get("Err"))

func _on_Button_button_up():
	var sensor_mac = $Control/MissionControl/Connection/MAC.text
	var response = sensor.connect(sensor_mac)
	if response.has("Ok"):
		connected = true
		$Control/MissionControl/Connection/ConnectionStatus.texture = load("res://assets/connected.png")
	else:
		print(response.get("Err"))
