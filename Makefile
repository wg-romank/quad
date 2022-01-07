gdnative:
	cd stm32-rust && cargo build --release

stm32:
	cd stm32-device && cargo flash --release --chip stm32f103C8

stm32-run:
	cargo embed --release

bluetooth-connect:
	sudo rfcomm connect rfcomm0 70:F2:09:01:65:00
