use crate::gpiote::*;
use drogue_device::{
    driver::led::{LEDMatrix, MatrixCommand},
    prelude::*,
};
use hal::gpio::{Input, Output, Pin, PullUp, PushPull};
use hal::pac::Interrupt;
use heapless::consts;
use nrf52833_hal as hal;

pub type Button = GpioteChannel<Pin<Input<PullUp>>>;
pub type LedMatrix = LEDMatrix<Pin<Output<PushPull>>, consts::U5, consts::U5>;

pub struct MyDevice {
    pub led: ActorContext<LedMatrix>,
    pub gpiote: InterruptContext<Gpiote>,
    pub btn_fwd: ActorContext<Button>,
    pub btn_back: ActorContext<Button>,
    pub activator: ActorContext<LedActivator>,
}

impl Device for MyDevice {
    fn start(&'static mut self, supervisor: &mut Supervisor) {
        let gpiote_addr = self.gpiote.start(supervisor);
        let fwd_addr = self.btn_fwd.start(supervisor);
        let back_addr = self.btn_back.start(supervisor);
        let matrix_addr = self.led.start(supervisor);
        let activator_addr = self.activator.start(supervisor);

        fwd_addr.subscribe(&gpiote_addr);
        back_addr.subscribe(&gpiote_addr);

        activator_addr.subscribe(&fwd_addr);
        activator_addr.notify(matrix_addr);
    }
}

pub struct LedActivator {
    pub matrix: Option<Address<LedMatrix>>,
}

impl Actor for LedActivator {
    type Event = ();
}

impl NotificationHandler<Address<LedMatrix>> for LedActivator {
    fn on_notification(&'static mut self, event: Address<LedMatrix>) -> Completion {
        self.matrix.replace(event);
        Completion::immediate()
    }
}

impl Sink<PinEvent> for LedActivator {
    fn notify(&self, event: PinEvent) {
        if let Some(matrix) = &self.matrix {
            match event {
                PinEvent(PinState::Low) => {
                    log::info!("Enabling led");
                    matrix.notify(MatrixCommand::On(0, 0));
                }
                _ => {
                    log::info!("Disabling led");
                    matrix.notify(MatrixCommand::Off(0, 0));
                }
            }
            matrix.notify(MatrixCommand::Render);
        }
    }
}
