extends Spatial


onready var sensor = Sensor.new()

var connected = false

var throttle_on = false
var throttle_value = 0.0

#func _ready():
#	var sensor_mac = $Control/VBoxContainer/HBoxContainer/TextEdit.text
#	sensor.connect(sensor_mac)
#	connected = true

func _physics_process(delta):
	if connected:
		pass
		#var tmp = sensor.get_angles()
		#if tmp != null:
		#	# TODO: check
		#	#$Paddle.rotate_x(tmp[0])
		#	#$Paddle.rotate_z(-tmp[1])
		#	$Paddle.rotation = Vector3(tmp[0], tmp[2], -tmp[1])

func _process(delta):
	pass
	#if connected:
	#	sensor.send_throttle(throttle_on, throttle_value)

func _input(event):
	if InputMap.event_is_action(event, "throttle"):
		throttle_value = event.get_axis_value()
	throttle_on =  Input.is_action_pressed("throttle_on")

	if connected:
		sensor.send_throttle(throttle_on, throttle_value)

func _on_Button_button_up():
	var sensor_mac = $Control/VBoxContainer/HBoxContainer/TextEdit.text
	sensor.connect(sensor_mac)
	connected = true
