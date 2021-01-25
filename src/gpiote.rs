use drogue_device::prelude::*;
use embedded_hal::digital::v2::InputPin;

use nrf52833_hal as hal;

use hal::gpiote::GpioteInputPin;

const NUM_CHANNELS: usize = 4;

pub struct Gpiote {
    gpiote: hal::gpiote::Gpiote,
}

pub struct GpioteChannel<P: GpioteInputPin> {
    channel: Channel,
    edge: Edge,
    pin: P,
}

impl<P: GpioteInputPin + Sized> Actor for GpioteChannel<P> {}

pub enum Edge {
    Rising,
    Falling,
    Both,
}

impl Gpiote {
    pub fn new(gpiote: hal::pac::GPIOTE) -> Self {
        let gpiote = hal::gpiote::Gpiote::new(gpiote);
        Self { gpiote }
    }

    pub fn configure_channel<P: GpioteInputPin>(
        &self,
        channel: Channel,
        pin: P,
        edge: Edge,
    ) -> GpioteChannel<P> {
        let ch = match channel {
            Channel::Channel0 => self.gpiote.channel0(),
            Channel::Channel1 => self.gpiote.channel1(),
            Channel::Channel2 => self.gpiote.channel2(),
            Channel::Channel3 => self.gpiote.channel3(),
        };

        let che = ch.input_pin(&pin);

        match edge {
            Edge::Rising => che.lo_to_hi(),
            Edge::Falling => che.hi_to_lo(),
            Edge::Both => che.toggle(),
        };

        che.enable_interrupt();
        GpioteChannel::new(channel, pin, edge)
    }
}

impl<P: GpioteInputPin> GpioteChannel<P> {
    pub fn new(channel: Channel, pin: P, edge: Edge) -> GpioteChannel<P> {
        GpioteChannel { channel, pin, edge }
    }
}

impl<P: GpioteInputPin> Sink<GpioteEvent> for GpioteChannel<P> {
    fn notify(&self, event: GpioteEvent) {
        match event {
            GpioteEvent(c) if c == self.channel => {
                log::info!("Channel {:?} notified!", self.channel);
            }
            _ => {}
        }
    }
}

/*
impl<P: GpioteInputPin> NotificationHandler<GpioteEvent> for GpioteChannel<P> {
    fn on_notification(&'static mut self, event: GpioteEvent) -> Completion {
        match event {
            GpioteEvent(c) if c == self.channel => {
                log::info!("Channel {:?} notified!", self.channel);
            }
            _ => {}
        }
        Completion::immediate()
    }
}*/

impl Interrupt for Gpiote {
    type Event = GpioteEvent;
    fn on_interrupt(&mut self, sink: &dyn Sink<Self::Event>) {
        if self.gpiote.channel0().is_event_triggered() {
            sink.notify(GpioteEvent(Channel::Channel0));
        }

        if self.gpiote.channel1().is_event_triggered() {
            sink.notify(GpioteEvent(Channel::Channel1));
        }

        if self.gpiote.channel2().is_event_triggered() {
            sink.notify(GpioteEvent(Channel::Channel2));
        }

        if self.gpiote.channel3().is_event_triggered() {
            sink.notify(GpioteEvent(Channel::Channel3));
        }
        self.gpiote.reset_events();
    }
}

impl Actor for Gpiote {}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum Channel {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
}

#[derive(Debug, Copy, Clone)]
pub struct GpioteEvent(pub Channel);

impl Message for GpioteEvent {}
