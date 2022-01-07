// #![deny(unsafe_code)]
#![no_std]
#![cfg_attr(not(doc), no_main)]

mod spatial;

use panic_rtt_target as _;

#[rtic::app(device = stm32f1xx_hal::pac, dispatchers = [WWDG])]
mod app {
    use nb;
    use nalgebra::Vector3;
    use rtt_target::{rprintln, rtt_init_print, UpChannel};

    use stm32f1xx_hal::device::USART1;
    use stm32f1xx_hal::dma::CircBuffer;

    use stm32f1xx_hal::serial;
    use stm32f1xx_hal::serial::Tx;
    use stm32f1xx_hal::{
        gpio::{
            gpiob::{PB6, PB7},
            Alternate, OpenDrain,
        },
        i2c::{BlockingI2c, DutyCycle, Mode},
        pac::I2C1,
        prelude::*,
        serial::{Config, Serial},
    };

    use systick_monotonic::*;

    use crate::spatial::SpatialOrientationDevice;
    use common::SpatialOrientation;
    use common::EOT;
    use common::BUFF_SIZE;

    use mpu6050::Mpu6050;

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>;

    type MPU = Mpu6050<BlockingI2c<I2C1, (PB6<Alternate<OpenDrain>>, PB7<Alternate<OpenDrain>>)>>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        recv: Option<CircBuffer<[u8; BUFF_SIZE], serial::RxDma1>>,
        usart1_tx: Tx<USART1>,
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

        let mut gpioa = dp.GPIOA.split();
        let pins = (
            gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh),
            gpioa.pa10,
        );

        let mut usart1 = Serial::usart1(
            dp.USART1,
            pins,
            &mut afio.mapr,
            Config::default().baudrate(9600.bps()),
            clocks,
        );
        usart1.listen(serial::Event::Idle);

        let dma1 = dp.DMA1.split();
        let (usart1_tx, rx) = usart1.split();
        let rrx = rx.with_dma(dma1.5);

        let buf = cortex_m::singleton!(: [[u8; BUFF_SIZE]; 2] = [[0; BUFF_SIZE]; 2]).unwrap();
        let rx_transfer = rrx.circ_read(buf);

        let mut gpiob = dp.GPIOB.split();
        let scl = gpiob.pb6.into_alternate_open_drain(&mut gpiob.crl);
        let sda = gpiob.pb7.into_alternate_open_drain(&mut gpiob.crl);

        let i2c1 = BlockingI2c::i2c1(
            dp.I2C1,
            (scl, sda),
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

        let mpu = Mpu6050::new(i2c1);

        mpu_init::spawn_after(1.secs(), mpu);

        (
            Shared {},
            Local {
                recv: Some(rx_transfer),
                usart1_tx,
            },
            init::Monotonics(mono),
        )
    }

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
        let tx: &mut serial::Tx<USART1> = cx.local.usart1_tx;
        let spawn_next_at = monotonics::now() + 4.micros();

        let raw_gyro = mpu.get_gyro().expect("unable to get gyro");
        let angles = mpu.get_acc_angles().expect("unable to get acc angles");

        s.adjust(raw_gyro - offset, angles);

        // rprintln!("{:?}", s);
        IntoIterator::into_iter(s.to_byte_array()).for_each(|byt| { nb::block!(tx.write(byt)).unwrap() });
        nb::block!(tx.write(EOT)).unwrap();

        gyro::spawn_at(spawn_next_at, mpu, offset, s);
    }

    // #[task(binds = USART1, local = [recv])]
    // fn on_rx(cx: on_rx::Context) {
    //     if let Some(rx) = cx.local.recv.take() {

    //         let (buf, mut rx) = rx.stop();
    //         let len = (buf[0].len() as u32 * 2) - rx.channel.ch().ndtr.read().bits();

    //         let (rx, channel) = rx.release();
    //         rx.clear_idle_interrupt();
    //         let rx = rx.with_dma(channel);

    //         cx.local.recv.replace(rx.circ_read(buf));
    //     }
    // }
}
