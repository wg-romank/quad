// #![deny(unsafe_code)]
#![no_std]
#![cfg_attr(not(doc), no_main)]

mod spatial;

use panic_rtt_target as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [WWDG, RTC])]
mod app {
    use nalgebra::Vector3;
    use rtt_target::{rprintln, rtt_init_print};

    use stm32f1xx_hal::device::USART2;
    use stm32f1xx_hal::dma::CircBuffer;
    use stm32f1xx_hal::gpio::{Output, PushPull, CRH};
    use stm32f1xx_hal::timer::{Timer, Tim4NoRemap, Event as TEvent};
    use stm32f1xx_hal::{
        gpio::{
            Pin,
            gpiob::{PB10, PB11},
            Alternate, OpenDrain,
        },
        i2c::{BlockingI2c, DutyCycle, Mode},
        pac::{I2C2, TIM4},
        prelude::*,
        pwm::{C1, C2, C3, C4, PwmChannel},
        serial::{Config, Serial, Tx, Event, RxDma2},
    };

    use systick_monotonic::*;

    use crate::spatial::{SpatialOrientationDevice, GYRO_FREQUENCY_HZ};
    use common::{SpatialOrientation, QuadState, MotorsMode};
    use common::COMMANDS_SIZE;
    use common::postcard::{from_bytes, to_vec};
    use common::heapless;



    use mpu6050::Mpu6050;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    type MPU = Mpu6050<BlockingI2c<I2C2, (PB10<Alternate<OpenDrain>>, PB11<Alternate<OpenDrain>>)>>;
    type PWM = (
        PwmChannel<TIM4, C1>,
        PwmChannel<TIM4, C2>,
        PwmChannel<TIM4, C3>,
        PwmChannel<TIM4, C4>,
    );

    #[shared]
    struct Shared {
        state: common::QuadState,
        mpu: Option<(MPU, Vector3<f32>, SpatialOrientation)>
    }

    #[local]
    struct Local {
        recv: Option<CircBuffer<[u8; COMMANDS_SIZE], RxDma2>>,
        usart2_tx: Tx<USART2>,
        pwm: PWM,
        led: Pin<Output<PushPull>, CRH, 'C', 13>,
        count: u32,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let dp = cx.device;
        let cp = cx.core;

        let mut flash = dp.FLASH.constrain();

        let rcc = dp.RCC.constrain();
        let mut afio = dp.AFIO.constrain();

        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let mono: MyMono = Systick::new(cp.SYST, clocks.sysclk().0);

        // BLUETOOTH
        let mut gpioa = dp.GPIOA.split();
        let usart2_pins = (
            gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl),
            gpioa.pa3,
        );

        let mut usart2 = Serial::usart2(
            dp.USART2,
            usart2_pins,
            &mut afio.mapr,
            Config::default().baudrate(9600.bps()),
            clocks,
        );
        usart2.listen(Event::Idle);

        let dma1 = dp.DMA1.split();
        let (usart2_tx, rx) = usart2.split();
        let rrx = rx.with_dma(dma1.6);

        let buf = cortex_m::singleton!(: [[u8; COMMANDS_SIZE]; 2] = [[0; COMMANDS_SIZE]; 2]).unwrap();
        let rx_transfer = rrx.circ_read(buf);

        // GYRO
        let mut gpiob = dp.GPIOB.split();
        let i2c_pins = (
            gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh),
            gpiob.pb11.into_alternate_open_drain(&mut gpiob.crh)
        );

        let i2c2 = BlockingI2c::i2c2(
            dp.I2C2,
            i2c_pins,
            Mode::Fast {
                frequency: 400_000.hz(),
                duty_cycle: DutyCycle::Ratio16to9,
            },
            clocks,
            1000,
            10,
            1000,
            1000,
        );

        // LED
        let mut gpioc = dp.GPIOC.split();
        let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        led.set_high();

        // TIMERS
        let mut telemetry_tim = Timer::tim2(dp.TIM2, &clocks)
                    .start_count_down(2.hz());
        telemetry_tim.listen(TEvent::Update);

        let mut gyro_tim = Timer::tim3(dp.TIM3, &clocks).start_count_down(GYRO_FREQUENCY_HZ.hz());
        gyro_tim.listen(TEvent::Update);

        // PWM
        let pb6 = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let pb7 = gpiob.pb7.into_alternate_push_pull(&mut gpiob.crl);
        let pb8 = gpiob.pb8.into_alternate_push_pull(&mut gpiob.crh);
        let pb9 = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

        let mut pwm = Timer::tim4(dp.TIM4, &clocks).pwm::<Tim4NoRemap, _, _, _>(
            (pb6, pb7, pb8, pb9), &mut afio.mapr, 200.hz()
        );
        let mut channels = pwm.split();
        channels.0.enable();
        channels.1.enable();
        channels.2.enable();
        channels.3.enable();


        let mpu = Mpu6050::new(i2c2);
        // the issue is that when board is offline i2c delay does not seem to work causing it to fail initialisation
        // todo: find a good way to handle it with i2c config
        mpu_init::spawn_after(1.secs(), mpu);

        (
            Shared {
                state: QuadState::default(),
                mpu: None
            },
            Local {
                recv: Some(rx_transfer),
                usart2_tx,
                pwm: channels,
                led,
                count: 0,
            },
            init::Monotonics(mono),
        )
    }

    #[task(shared = [mpu])]
    fn mpu_init(mut cx: mpu_init::Context, mut mpu: MPU) {
        cx.shared.mpu.lock(|m| {
            mpu.init().expect("unable to init MPU6050");
            let offset = (0..2000)
                .flat_map(|_| mpu.get_gyro().ok())
                .reduce(|l, r| (l + r) / 2.0)
                .expect("no calibration measurements");
            let angles = mpu.get_acc_angles().expect("unable to get acc angles");

            let spatial_orientation = SpatialOrientation::new(angles);

            *m = Some((mpu, offset, spatial_orientation))
        })
    }

    #[task(binds = TIM3, local = [pwm, led], shared = [mpu, state], priority = 1)]
    fn gyro(mut cx: gyro::Context) {
        let mpu = cx.shared.mpu;
        let state = cx.shared.state;

        (mpu, state).lock(|m, state| {
            if let Some((mpu, offset, s)) = m {
                let raw_gyro = mpu.get_gyro().expect("unable to get gyro");
                let angles = mpu.get_acc_angles().expect("unable to get acc angles");

                s.adjust(raw_gyro - *offset, angles);
                rprintln!("{:?}", s);

                let t = state.throttle;
                let led_on = state.led;
                let mode = &state.mode;
                let stab_on = state.stabilisation;

                let [dx1, dx2, dx3, dx4] = if stab_on {
                    s.compute_corrections(&state.desired_orientation)
                } else {
                    [0., 0., 0., 0.]
                };

                let max_duty: f32 = u16::MAX as f32;
                match mode {
                    MotorsMode::All => {
                        cx.local.pwm.3.set_duty((max_duty * (t + dx1)) as u16);
                        cx.local.pwm.2.set_duty((max_duty * (t + dx2)) as u16);
                        cx.local.pwm.1.set_duty((max_duty * (t + dx3)) as u16);
                        cx.local.pwm.0.set_duty((max_duty * (t + dx4)) as u16);
                    },
                    MotorsMode::X1 => {
                        cx.local.pwm.3.set_duty((max_duty * (t + dx1)) as u16);
                        cx.local.pwm.2.set_duty(0);
                        cx.local.pwm.1.set_duty(0);
                        cx.local.pwm.0.set_duty(0);
                    },
                    MotorsMode::X2 => {
                        cx.local.pwm.3.set_duty(0);
                        cx.local.pwm.2.set_duty((max_duty * (t + dx2)) as u16);
                        cx.local.pwm.1.set_duty(0);
                        cx.local.pwm.0.set_duty(0);
                    },
                    MotorsMode::X3 => {
                        cx.local.pwm.3.set_duty(0);
                        cx.local.pwm.2.set_duty(0);
                        cx.local.pwm.1.set_duty((max_duty * (t + dx3)) as u16);
                        cx.local.pwm.0.set_duty(0);
                    },
                    MotorsMode::X4 => {
                        cx.local.pwm.3.set_duty(0);
                        cx.local.pwm.2.set_duty(0);
                        cx.local.pwm.1.set_duty(0);
                        cx.local.pwm.0.set_duty((max_duty * (t + dx4)) as u16);
                    }
                }

                if led_on {
                    cx.local.led.set_low();
                } else {
                    cx.local.led.set_high();
                }
            }
        });
    }

    #[task(binds = TIM2, local = [usart2_tx], shared = [state], priority = 1)]
    fn telemetry(mut cx: telemetry::Context) {
        // todo:
        cx.shared.state.lock(|s| {
            let tx: &mut Tx<USART2> = cx.local.usart2_tx;
            let packet: heapless::Vec<u8, 30> = to_vec(s).expect("unable to serialize to buff");
            IntoIterator::into_iter(packet)
                .for_each(|byt| { nb::block!(tx.write(byt)).unwrap() });
            rprintln!("State sent")
        })

        // cx.shared.state.lock(|state| {
        //     rprintln!("T {:?}", state);
        // });
    }

    #[task(binds = USART2, local = [recv], shared = [state], priority = 2)]
    fn on_rx(mut cx: on_rx::Context) {
        if let Some(rx) = cx.local.recv.take() {
            let (buf, mut rx) = rx.stop();

            if let Ok(command) = from_bytes(&buf[0]) {
                cx.shared.state.lock(|state| state.update(command));
            }

            let (rx, channel) = rx.release();
            rx.clear_idle_interrupt();
            let rx = rx.with_dma(channel);

            cx.local.recv.replace(rx.circ_read(buf));
        }
    }
}
