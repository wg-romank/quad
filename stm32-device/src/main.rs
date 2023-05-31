// #![deny(unsafe_code)]
#![no_std]
#![cfg_attr(not(doc), no_main)]

mod spatial;

use panic_rtt_target as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [WWDG])]
mod app {
    use nalgebra::Vector3;
    use rtt_target::{rprintln, rtt_init_print};

    use stm32f1xx_hal::device::USART2;
    use stm32f1xx_hal::dma::CircBuffer;
    use stm32f1xx_hal::timer::{Timer, Tim4NoRemap, CountDownTimer, Event as TEvent};
    use stm32f1xx_hal::{
        gpio::{
            gpiob::{PB10, PB11},
            Alternate, OpenDrain,
        },
        i2c::{BlockingI2c, DutyCycle, Mode},
        pac::{I2C2, TIM2, TIM4},
        prelude::*,
        pwm::{C1, C2, C3, C4, PwmChannel},
        serial::{Config, Serial, Tx, Event, RxDma2},
    };

    use systick_monotonic::*;

    use crate::spatial::SpatialOrientationDevice;
    use common::{SpatialOrientation, Command};
    use common::COMMAND_SIZE;

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
    struct Shared {}

    #[local]
    struct Local {
        recv: Option<CircBuffer<[u8; COMMAND_SIZE], RxDma2>>,
        usart2_tx: Tx<USART2>,
        pwm: PWM,
        pwm_tim: CountDownTimer<TIM2>,
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

        let buf = cortex_m::singleton!(: [[u8; COMMAND_SIZE]; 2] = [[0; COMMAND_SIZE]; 2]).unwrap();
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

        // PWM
        let mut pwm_tim = Timer::tim2(dp.TIM2, &clocks)
                    .start_count_down(1.hz());
        pwm_tim.listen(TEvent::Update);

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
        mpu_init::spawn_after(1.secs(), mpu);

        (
            Shared {},
            Local {
                recv: Some(rx_transfer),
                usart2_tx,
                pwm: channels,
                pwm_tim,
                count: 0,
            },
            init::Monotonics(mono),
        )
    }

    // testing PWM
    // #[task(binds = TIM2, local = [count, pwm, pwm_tim], priority = 2)]
    // fn motors(cx: motors::Context) {
    //     rprintln!("TIM TRIGGER");
    //     if *cx.local.count % 2 == 0 {
    //         cx.local.pwm.0.set_duty(0);
    //         cx.local.pwm.1.set_duty(0);
    //         cx.local.pwm.2.set_duty(0);
    //         cx.local.pwm.3.set_duty(0);
    //         rprintln!("DUTY ZERO");
    //     } else {
    //         // let max_duty = (1.0 * cx.local.pwm.0.get_max_duty() as f32) as u16;
    //         let max_duty = u16::MAX;
    //         cx.local.pwm.0.set_duty(max_duty);
    //         cx.local.pwm.1.set_duty(max_duty);
    //         cx.local.pwm.2.set_duty(max_duty);
    //         cx.local.pwm.3.set_duty(max_duty);
    //         rprintln!("DUTY MAX");
    //     }
    //     *cx.local.count += 1;

    //     cx.local.pwm_tim.clear_update_interrupt_flag();
    //     rprintln!("INTERRUPT CLEAR");
    // }

    #[task]
    fn mpu_init(_: mpu_init::Context, mut mpu: MPU) {
        mpu.init().expect("unable to init MPU6050");

        let offset = (0..2000)
            .flat_map(|_| mpu.get_gyro().ok())
            .reduce(|l, r| (l + r) / 2.0)
            .expect("no calibration measurements");
        let angles = mpu.get_acc_angles().expect("unable to get acc angles");

        let spatial_orientation = SpatialOrientation::new(angles);

        gyro::spawn(mpu, offset, spatial_orientation);
    }

    #[task(local = [usart2_tx], capacity = 1)]
    fn gyro(cx: gyro::Context, mut mpu: MPU, offset: Vector3<f32>, mut s: SpatialOrientation) {
        let tx: &mut Tx<USART2> = cx.local.usart2_tx;
        let spawn_next_at = monotonics::now() + 4.micros();

        let raw_gyro = mpu.get_gyro().expect("unable to get gyro");
        let angles = mpu.get_acc_angles().expect("unable to get acc angles");

        s.adjust(raw_gyro - offset, angles);

        rprintln!("{:?}", s);
        // IntoIterator::into_iter(s.to_byte_array())
        //     .for_each(|byt| { nb::block!(tx.write(byt)).unwrap() });
        // nb::block!(tx.write(EOT)).unwrap();

        gyro::spawn_at(spawn_next_at, mpu, offset, s);
    }

    // todo: restore
    #[task(binds = USART2, local = [recv, pwm], priority = 2)]
    fn on_rx(cx: on_rx::Context) {
        if let Some(rx) = cx.local.recv.take() {
            let (buf, mut rx) = rx.stop();
            let len = (buf[0].len() as u32 * 2) - rx.channel.ch().ndtr.read().bits();

            let command = Command::from_byte_slice(&buf[0]);
            rprintln!("got {:?}", command);

            if command.throttle <= 1.0 && command.throttle >= 0.0 {
                let max_duty: u16 = u16::MAX;
                let duty = (max_duty as f32 * command.throttle) as u16;
                cx.local.pwm.0.set_duty(duty);
                cx.local.pwm.1.set_duty(duty);
                cx.local.pwm.2.set_duty(duty);
                cx.local.pwm.3.set_duty(duty);
                rprintln!("duty {}", duty);
            }

            let (rx, channel) = rx.release();
            rx.clear_idle_interrupt();
            let rx = rx.with_dma(channel);

            cx.local.recv.replace(rx.circ_read(buf));
        }
    }
}
