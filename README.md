# stm32-quad

Drone in the making

# Hardware

1. Main board - stm32f103c8
2. IMU with Gyro and Accelerometer - mpu6050
3. Telemetry and control via Bluetooth - JDY-31-SPP
4. Power delivery - YYNMOS-4 MOSFET driver
5. Battery - Syma X5C 3.7V 600mah or any other rated 25C
6. Motors - 4 x 8520 coreless motor
7. Propellers - 4 x 55mm (2 x CW + 2 x CCW), shaft diameter must match of motors
8. [Frame](resources/part.stl) - 3d printed in 4 identical pieces, might need to adjust tolerances

Total weight assembled ~ 82g

# Wiring diagram

<img src="resources/wiring.svg">

Power for bluetooth module and IMU can be connected to main board directly for convenience.

`GNDX` terminals on mosfet driver can be soldered together and connected to `GND` on main board.

Important note is since motors need to spin in opposite directions, interleaving one anonther one needs to pay attention connecting motor wires to mosfet driver.

Motors polarity described as `+` and `-` and is reversed for 2nd and 4th motor respectively.

Battery terminals are shared betewen `DC+/-` connections on mosfet driver and `3V/GND` on main board.

# Motors & IMU orientation

<TODO>

# Notes

Some measurements, using 4-channel MOSFET driver (YYNMOS-4), with 1000mah standard battery peak single motor amps are around 0.8. With '25c' battery around 1.4.

Very loose measurements for actual lift around 10g for single motor which is way too low comparing with the internet. e.g. https://www.youtube.com/watch?v=AMWXXCHrHto but could be also due to leverage (only single motor pulling)

Possibly have to change battery/motors or both. Good battery reference seems to be Nanotech (e.g. Nanotech 750mah)

Kingkong 65mm seems to be good fit wiht 8520s [on this rc forum](https://www.rcgroups.com/forums/showthread.php?2811110-Racerstar-8520-coreless-motors-Review-Thrust-Test-(8-5x20mm-brushed-motors)/page2)

# Todos

- [ ] Test the rig current consumption with 2 and 4 motors simultaneously
- [ ] Figure out how to strap motor on to mosfet driver pcb as frame does not seems to fit it anymore