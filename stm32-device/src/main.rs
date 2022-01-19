// #![deny(unsafe_code)]
#![no_std]
#![cfg_attr(not(doc), no_main)]

mod spatial;

use panic_rtt_target as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [WWDG])]
mod app {
    use nb;
    use nalgebra::Vector3;
    use rtt_target::{rprintln, rtt_init_print, UpChannel, rprint};

    use stm32f1xx_hal::device::USART1;
    use stm32f1xx_hal::dma::CircBuffer;
    use stm32f1xx_hal::timer::{Tim2NoRemap, Timer, Tim4NoRemap, Event as TEvent, CountDownTimer};
    use stm32f1xx_hal::{
        gpio::{
            gpiob::{PB6, PB7}, CRH,
            Alternate, OpenDrain, Pin, PushPull, Output
        },
        i2c::{BlockingI2c, DutyCycle, Mode},
        pac::{I2C1, TIM1, TIM2, TIM3, TIM4},
        prelude::*,
        pwm::{C3, Channel, Pwm},
        serial::{Config, Serial, Tx, Event, RxDma1},
    };

    use systick_monotonic::*;

    use crate::spatial::SpatialOrientationDevice;
    use common::{SpatialOrientation, Command};
    use common::EOT;
    use common::COMMAND_SIZE;

    use mpu6050::Mpu6050;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    type MPU = Mpu6050<BlockingI2c<I2C1, (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>)>>;
    type PWM = Pwm<TIM4, Tim4NoRemap, C3, Pin<Alternate<PushPull>, CRH, 'B', 8_u8>>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        recv: Option<CircBuffer<[u8; COMMAND_SIZE], RxDma1>>,
        usart1_tx: Tx<USART1>,
        pwm: PWM,
        count: u32,
        pwm_tim: CountDownTimer<TIM2>,
        en12: Pin<Output<PushPull>, CRH, 'B', 9_u8>,
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
        let usart1_pins = (
            gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh),
            gpioa.pa10,
        );

        let mut usart1 = Serial::usart1(
            dp.USART1,
            usart1_pins,
            &mut afio.mapr,
            Config::default().baudrate(9600.bps()),
            clocks,
        );
        usart1.listen(Event::Idle);

        let dma1 = dp.DMA1.split();
        let (usart1_tx, rx) = usart1.split();
        let rrx = rx.with_dma(dma1.5);

        let buf = cortex_m::singleton!(: [[u8; COMMAND_SIZE]; 2] = [[0; COMMAND_SIZE]; 2]).unwrap();
        let rx_transfer = rrx.circ_read(buf);

        // GYRO
        let mut gpiob = dp.GPIOB.split();
        let i2c_pins = (
            gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl),
            gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl)
        );

        let i2c1 = BlockingI2c::i2c1(
            dp.I2C1,
            i2c_pins,
            &mut afio.mapr,
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
        let mut en12 = gpiob.pb9.into_push_pull_output(&mut gpiob.crh);

        let mot1 = gpiob.pb8.into_alternate_push_pull(&mut gpiob.crh);

        let mut pwm_tim = Timer::tim2(dp.TIM2, &clocks)
            .start_count_down(1.hz());
        pwm_tim.listen(TEvent::Update);

        let mut pwm = Timer::tim4(dp.TIM4, &clocks).pwm::<Tim4NoRemap, _, _, _>(mot1, &mut afio.mapr, 1.khz());
        pwm.enable(Channel::C3);

        //
        let mpu = Mpu6050::new(i2c1);
        mpu_init::spawn_after(1.secs(), mpu);

        (
            Shared {},
            Local {
                recv: Some(rx_transfer),
                usart1_tx,
                pwm,
                count: 0,
                pwm_tim,
                en12,
            },
            init::Monotonics(mono),
        )
    }

    // #[task(binds = TIM2, local = [count, pwm, pwm_tim], priority = 2)]
    // fn motors(cx: motors::Context) {
    //     rprintln!("TIM TRIGGER");
    //     if *cx.local.count % 2 == 0 {
    //         let max_duty = cx.local.pwm.get_max_duty();
    //         cx.local.pwm.set_duty(Channel::C3, max_duty);
    //         rprintln!("DUTY MAX");
    //     } else {
    //         cx.local.pwm.set_duty(Channel::C3, 0);
    //         rprintln!("DUTY ZERO");
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

    #[task(local = [usart1_tx], capacity = 1)]
    fn gyro(cx: gyro::Context, mut mpu: MPU, offset: Vector3<f32>, mut s: SpatialOrientation) {
        let tx: &mut Tx<USART1> = cx.local.usart1_tx;
        let spawn_next_at = monotonics::now() + 4.micros();

        let raw_gyro = mpu.get_gyro().expect("unable to get gyro");
        let angles = mpu.get_acc_angles().expect("unable to get acc angles");

        s.adjust(raw_gyro - offset, angles);

        // rprintln!("{:?}", s);
        IntoIterator::into_iter(s.to_byte_array()).for_each(|byt| { nb::block!(tx.write(byt)).unwrap() });
        nb::block!(tx.write(EOT)).unwrap();

        gyro::spawn_at(spawn_next_at, mpu, offset, s);
    }

    #[task(binds = USART1, local = [recv, pwm, en12], priority = 2)]
    fn on_rx(cx: on_rx::Context) {
        if let Some(rx) = cx.local.recv.take() {
            let (buf, mut rx) = rx.stop();
            let len = (buf[0].len() as u32 * 2) - rx.channel.ch().ndtr.read().bits();

            let command = Command::from_byte_slice(&buf[0]);
            rprintln!("got {:?}", command);

            // todo: find a better way
            // workaround malformed packet
            if command.throttle_on {
                cx.local.en12.set_high();
            } else {
                cx.local.en12.set_low();
            }

            if command.throttle <= 1.0 && command.throttle >= 0.0 {
                let max_duty = cx.local.pwm.get_max_duty();
                let duty = (max_duty as f32 * command.throttle) as u16;
                cx.local.pwm.set_duty(Channel::C3, duty);
                rprintln!("duty {}", duty);
            }

            let (rx, channel) = rx.release();
            rx.clear_idle_interrupt();
            let rx = rx.with_dma(channel);

            cx.local.recv.replace(rx.circ_read(buf));
        }
    }
}
