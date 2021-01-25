#![no_main]
#![no_std]

mod device;
mod gpiote;

use panic_halt as _;

use core::sync::atomic::{compiler_fence, Ordering};
use cortex_m_rt::{entry, exception};
use drogue_device::prelude::*;
use embedded_hal::digital::v2::OutputPin;
use log::LevelFilter;
use rtt_logger::RTTLogger;
use rtt_target::rtt_init_print;

use nrf52833_hal as hal;

use crate::device::*;
use crate::gpiote::*;

static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Trace);

#[entry]
fn main() -> ! {
    rtt_init_print!();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let mut device = hal::pac::Peripherals::take().unwrap();

    let port0 = hal::gpio::p0::Parts::new(device.P0);
    let port1 = hal::gpio::p1::Parts::new(device.P1);

    let clocks = hal::clocks::Clocks::new(device.CLOCK).enable_ext_hfosc();
    let _clocks = clocks.start_lfclk();

    let gpiote = Gpiote::new(device.GPIOTE);
    let button_fwd: Button = gpiote.configure_channel(
        Channel::Channel0,
        port0.p0_14.into_pullup_input().degrade(),
        Edge::Falling,
    );
    let button_back: Button = gpiote.configure_channel(
        Channel::Channel1,
        port0.p0_23.into_pullup_input().degrade(),
        Edge::Falling,
    );

    let device = MyDevice {
        btn_fwd: ActorContext::new(button_fwd),
        btn_back: ActorContext::new(button_back),
        gpiote: InterruptContext::new(gpiote, hal::pac::Interrupt::GPIOTE),
    };

    device!( MyDevice = device; 1024 );
}
