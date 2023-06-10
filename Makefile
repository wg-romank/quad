gdnative:
	cd client-util && cargo build --release

flash:
	cd flight-controller && cargo flash --release --chip stm32f103C8

embed:
	cd flight-controller && cargo embed --release

bluetooth-connect:
	sudo rfcomm connect rfcomm0 70:F2:09:01:65:00
