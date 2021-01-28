use crate::gpiote::*;
use drogue_device::{
    driver::led::{LEDMatrix, MatrixCommand},
    prelude::*,
};
use hal::gpio::{Input, Output, Pin, PullUp, PushPull};
use hal::pac::Interrupt;
use heapless::consts;
use nrf52833_hal as hal;

pub type Button = GpioteChannel<MyDevice, Pin<Input<PullUp>>>;
pub type LedMatrix = LEDMatrix<Pin<Output<PushPull>>, consts::U5, consts::U5>;

pub struct MyDevice {
    pub led: ActorContext<Self, LedMatrix>,
    pub gpiote: InterruptContext<Self, Gpiote<Self>>,
    pub btn_fwd: ActorContext<Self, Button>,
    pub btn_back: ActorContext<Self, Button>,
}

impl Device for MyDevice {
    fn mount(&'static mut self, bus: &EventBus<Self>, supervisor: &mut Supervisor) {
        let _gpiote_addr = self.gpiote.mount(bus, supervisor);
        let _fwd_addr = self.btn_fwd.mount(bus, supervisor);
        let _back_addr = self.btn_back.mount(bus, supervisor);
        let _matrix_addr = self.led.mount(bus, supervisor);
    }
}

impl EventConsumer<GpioteEvent> for MyDevice {
    fn on_event(&'static mut self, event: GpioteEvent) {
        self.btn_fwd.address().notify(event);
    }
}

impl EventConsumer<PinEvent> for MyDevice {
    fn on_event(&'static mut self, event: PinEvent) {
        log::info!("Got pin event LOL");
    }
}

/*
impl Actor for MyDevice {}

impl NotificationHandler<GpioteEvent> for MyDevice {
    fn on_notification(&'static mut self, event: GpioteEvent) -> Completion {
        Completion::immediate()
    }
}

*/
