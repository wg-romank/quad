31/05/23
========
- arrived 5x stm32 boards, swapping board seem to fix previous issues with flashing
- had a weird issue of gyro task not spawning after switching uart bluetooth pins
- switched off the board gave it a rest & the issue is not observed anymore
- tested with 25c battery, seems too weak to lift off, but did hover a little
- battery showed 3v with multimeter - probably almost discharged
- unable to check how much current drawn in total, seems multimeter is having a cap of 1a or just above that
- made a little harness out of thread to do stabilization; pretty fun to fire off the motors

todo:
- fully charge battery & re-do max thrust test
- order a different battery with more punch
- could start to implement PID with thread harness already

08/06/23
========
- testing thrust with fully charged battery showed promising results
- found issue in wiring forcing one of the motors to spin in wrong direction (:O)
- restructured commands format to accomodate for different types of actions, not just throttle
  - included `borsh` to automatically derive encoders / decoders
- added commands to enable / disable main board led and stabilisation
- shorted battery (T_T)
- implemented naive stabilisation
  - found issues in implementation since IMU is upside down and some of axises are flipped

todo:
- write a page on how naive stabilisation works
- check stabilisation effect on light throttle
- implement moving pitch / roll with gamepad axis for control

11/06/23
========
- fixed stabilisation direction, seems to be engaging correct motors based on tilt now
- fixed wiring order & updated flight controller pwm code
- shorted battery again (X_X)
- restructured project a bit with better namings
- added testing mode for motors to only enable single one
  - currently it runs in order 1,2,3,4 then enabling all

todo:
- motors seem to draw power unevenly (could be due to limited current of damaged battery)
- get a new battery and check current for all motors in single mode (possible with limiting current on mutlimeter?)
- motor can stuck in 'on' mode even when not enabled (parasitic capacitance?)
  - turning motor on again seems to unstuck them (could be solved when undamaged battery is installed?)

14/06/23
========
- crashed drone while testing power output with fresh and fully charged battery
- one of props is slighly bent
- doulbe check & triple-check the battery connection when charging
- some issues found with sending commands to quad, probably this is causing motors to keep spinning and other stuff
- uneven power improves when graudally increasing throttle, probably control task is running at too high frequency so mosfet driver does not have enough time to respond
- re-watched talk on flight controller stabilisation, missing part is PI controller for rate of change for gyros, not just angles

todos:
- improve communication to limit number of messages sent and ensure commands do not clog the queue
- reduce control frequency, possibly split gyro into separate task to do median filtering on acc with high frequency
- implement heartbeat & throttle cutoff for safety