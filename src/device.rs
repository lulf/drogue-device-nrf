use crate::gpiote::*;
use drogue_device::prelude::*;
use hal::gpio::{Input, Pin, PullUp};
use hal::pac::Interrupt;
use nrf52833_hal as hal;

pub type Button = GpioteChannel<Pin<Input<PullUp>>>;

pub struct MyDevice {
    pub gpiote: InterruptContext<Gpiote>,
    pub btn_fwd: ActorContext<Button>,
    pub btn_back: ActorContext<Button>,
}

impl Device for MyDevice {
    fn start(&'static mut self, supervisor: &mut Supervisor) {
        let gpiote_addr = self.gpiote.start(supervisor);
        let fwd_addr = self.btn_fwd.start(supervisor);
        let back_addr = self.btn_back.start(supervisor);

        fwd_addr.subscribe(&gpiote_addr);
        back_addr.subscribe(&gpiote_addr);
    }
}
