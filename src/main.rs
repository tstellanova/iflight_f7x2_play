#![no_main]
#![no_std]

use cortex_m_rt as rt;
use rt::entry;

use panic_rtt_core::{self, rprintln, rtt_init_print};

use icm20689::SpiInterface;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::ToggleableOutputPin;

const IMU_REPORTING_RATE_HZ: u16 = 200;
const IMU_REPORTING_INTERVAL_MS: u16 = (1000 / IMU_REPORTING_RATE_HZ);


#[cfg(feature = "stm32f7x")]
mod peripherals_stm32f7x;
#[cfg(feature = "stm32f7x")]
use peripherals_stm32f7x as peripherals;

fn main() {
    rprintln!("--- bEGIN --- ");

    let (stuff) = peripherals::setup_peripherals();

    let spi_bus1 = shared_bus::CortexMBusManager::new(spi1_port);


    // TODO troubleshoot icm20689  -- probe is consistently failing
    let mut tdk_6dof = icm20689::Builder::new_spi(spi_bus1.acquire(), spi1_cs_tdk);
    if tdk_6dof.setup(&mut delay_source).is_err() {
        console_print(&mut po_tx, format_args!("icm20689 failed\r\n"));
    }

}
