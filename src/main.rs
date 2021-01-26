#![no_main]
#![no_std]

mod device;
mod gpiote;

use panic_halt as _;

use core::sync::atomic::{compiler_fence, Ordering};
use cortex_m_rt::{entry, exception};
use drogue_device::{driver::led::LEDMatrix, prelude::*};
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::{Input, Level, Output, Pin, PullUp, PushPull};
use heapless::{consts, Vec};
use log::LevelFilter;
use rtt_logger::RTTLogger;
use rtt_target::rtt_init_print;

use nrf52833_hal as hal;

use crate::device::*;
use crate::gpiote::*;

static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Info);

#[entry]
fn main() -> ! {
    rtt_init_print!();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info);

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

    let mut rows = Vec::<_, consts::U5>::new();
    rows.push(port0.p0_21.into_push_pull_output(Level::Low).degrade())
        .ok();
    rows.push(port0.p0_22.into_push_pull_output(Level::Low).degrade())
        .ok();
    rows.push(port0.p0_15.into_push_pull_output(Level::Low).degrade())
        .ok();
    rows.push(port0.p0_24.into_push_pull_output(Level::Low).degrade())
        .ok();
    rows.push(port0.p0_19.into_push_pull_output(Level::Low).degrade())
        .ok();

    let mut cols = Vec::<_, consts::U5>::new();
    cols.push(port0.p0_28.into_push_pull_output(Level::Low).degrade())
        .ok();
    cols.push(port0.p0_11.into_push_pull_output(Level::Low).degrade())
        .ok();
    cols.push(port0.p0_31.into_push_pull_output(Level::Low).degrade())
        .ok();
    cols.push(port1.p1_05.into_push_pull_output(Level::Low).degrade())
        .ok();
    cols.push(port0.p0_30.into_push_pull_output(Level::Low).degrade())
        .ok();

    let led = LEDMatrix::<Pin<Output<PushPull>>, consts::U5, consts::U5>::new(rows, cols);

    let device = MyDevice {
        btn_fwd: ActorContext::new(button_fwd),
        btn_back: ActorContext::new(button_back),
        gpiote: InterruptContext::new(gpiote, hal::pac::Interrupt::GPIOTE),
        led: ActorContext::new(led),
        activator: ActorContext::new(LedActivator { matrix: None }),
    };

    device!( MyDevice = device; 1024 );
}
