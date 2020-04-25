use stm32f7xx_hal as p_hal;

use p_hal::device as pac;
use embedded_hal as ehal;

// use p_hal::flash::FlashExt;
use ehal::blocking::delay::DelayMs;
use ehal::digital::v2::{OutputPin, ToggleableOutputPin};
use p_hal::gpio::GpioExt;
use p_hal::rcc::RccExt;
use p_hal::time::{Hertz, U32Ext};


use icm20689::SpiInterface;
use stm32f7xx_hal::delay::Delay;

//MCU: STM32F722RET6
// IMU1: ICM20689_A (SPI1)
// IMU2: ICM20689_B (SPI1)
// OSD: AT7456E  (MAXIM MAX7456) (SPI2)
// Barometer: BMP280 (SPI3)
// Blackbox: Micron M25P16VP (SPI3)



pub fn setup_peripherals() -> (u32, Delay, u32, u32,
    //SPI1
    impl ehal::blocking::spi::Transfer<u8, Error = p_hal::spi::Error>
    + ehal::blocking::spi::Write<u8, Error = p_hal::spi::Error>,
   // ICM20689 CS1
   impl OutputPin<Error = p_hal::Never>,

) {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Set up the system clock
    let rcc = dp.RCC.constrain();
    // HSI: use default internal oscillator
    //let clocks = rcc.cfgr.freeze();
    // HSE: external crystal oscillator must be connected
    // let clocks = rcc
    //     .cfgr
    //     .use_hse(8.mhz()) //f4 discovery board has 8 MHz crystal for HSE
    //     .sysclk(128.mhz())
    //     .pclk1(48.mhz())
    //     // .pclk2(48.mhz())
    //     .freeze();

    let clocks = rcc
        .cfgr
        .use_hse(25.mhz())  //TODO verify crystal
        .sysclk(216.mhz())
        // .pclk1(48.mhz())
        // .pclk2(48.mhz())
        .freeze();

    // let mut ccdr = rcc.freeze(vos, &dp.SYSCFG);


    let delay_source = p_hal::delay::Delay::new(cp.SYST, clocks);

    // let hclk = clocks.hclk();
    // let rng_clk = clocks.pll48clk().unwrap_or(0u32.hz());
    // let pclk1 = clocks.pclk1();
    // d_println!(get_debug_log(), "hclk: {} /16: {} pclk1: {} rng_clk: {}", hclk.0, hclk.0 / 16, pclk1.0, rng_clk.0);

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    // let gpiod = dp.GPIOD.split();

    let user_led1 = gpioc.pc13.into_push_pull_output();
    // let user_led1 = gpiod.pd12.into_push_pull_output(); //f4discovery

    // resource SPI_SCK 1 A05
    // resource SPI_SCK 2 B13
    // resource SPI_SCK 3 B03
    // resource SPI_MISO 1 A06
    // resource SPI_MISO 2 B14
    // resource SPI_MISO 3 B04
    // resource SPI_MOSI 1 A07
    // resource SPI_MOSI 2 B15
    // resource SPI_MOSI 3 B05

    //setup SPI1 for the bulk of SPI-connected internal sensors
    // TODO need to increase SPI1 clock speed?
    let spi1_port = {
        let sck = gpioa.pa5.into_alternate_af5();
        let miso = gpioa.pa6.into_alternate_af5();
        let mosi = gpiod.pd7.into_alternate_af5();
        dp.SPI1
            .spi((sck, miso, mosi), embedded_hal::spi::MODE_3, 2.mhz(), &ccdr)
    };



    //PF2 is CS for TDK ICM20689 (2 MHz - 8 MHz)
    let mut spi1_cs_tdk = gpiof
        .pf2
        .into_push_pull_output()
        .set_speed(p_hal::gpio::Speed::Low); //TODO verify: should be 2 MHz
    spi1_cs_tdk.set_high().unwrap();


    (user_led1, delay_source, spi1_port, spi1_cs_tdk)
}


/// Hello
///
pub type Spi1PortType = p_hal::spi::Spi<
    pac::SPI1,
    (
        p_hal::gpio::gpioa::PA5<p_hal::gpio::Alternate<p_hal::gpio::AF5>>, //SCLK
        p_hal::gpio::gpioa::PA6<p_hal::gpio::Alternate<p_hal::gpio::AF5>>, //MISO
        p_hal::gpio::gpioa::PA7<p_hal::gpio::Alternate<p_hal::gpio::AF5>>, //MOSI
    ),
>;

type ChipSelectPinType =
    p_hal::gpio::gpioa::PA15<p_hal::gpio::Output<p_hal::gpio::OpenDrain>>; //CSN
type HIntPinType =
    p_hal::gpio::gpiob::PB0<p_hal::gpio::Input<p_hal::gpio::PullUp>>; //HINTN
type WakePinType =
    p_hal::gpio::gpiob::PB1<p_hal::gpio::Output<p_hal::gpio::OpenDrain>>; //PushPull>>; // WAKE
type ResetPinType =
    p_hal::gpio::gpiob::PB10<p_hal::gpio::Output<p_hal::gpio::OpenDrain>>; // RESET

pub type BnoSpi1Lines = SpiControlLines<
    Spi1PortType,
    ChipSelectPinType,
    HIntPinType,
    WakePinType,
    ResetPinType,
>;
