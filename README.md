# stm32-quad

Drone in the making

# Notes

Some measurements, using 4-channel MOSFET driver (YYNMOS-4), with 1000mah standard battery peak single motor amps are around 0.8. With '25c' battery around 1.4.

Very loose measurements for actual lift around 10g for single motor which is way too low comparing with the internet. e.g. https://www.youtube.com/watch?v=AMWXXCHrHto but could be also due to leverage (only single motor pulling)

Possibly have to change battery/motors or both. Good battery reference seems to be Nanotech (e.g. Nanotech 750mah)

Kingkong 65mm seems to be good fit wiht 8520s [on this rc forum](https://www.rcgroups.com/forums/showthread.php?2811110-Racerstar-8520-coreless-motors-Review-Thrust-Test-(8-5x20mm-brushed-motors)/page2)

# Todos

- [ ] Test the rig current consumption with 2 and 4 motors simultaneously
- [ ] Figure out how to strap motor on to mosfet driver pcb as frame does not seems to fit it anymore