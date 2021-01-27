use drogue_device::prelude::*;
use embedded_hal::digital::v2::InputPin;

use nrf52833_hal as hal;

use hal::gpiote::GpioteInputPin;

const NUM_CHANNELS: usize = 4;

pub struct Gpiote<D: Device + EventHandler<GpioteEvent>> {
    gpiote: hal::gpiote::Gpiote,
    broker: Option<Broker<D>>,
}

pub struct GpioteChannel<D: Device + EventHandler<PinEvent>, P: InputPin + GpioteInputPin + 'static>
{
    broker: Option<Broker<D>>,
    channel: Channel,
    edge: Edge,
    pin: P,
}

impl<D: Device + EventHandler<PinEvent>, P: InputPin + GpioteInputPin + Sized> Actor
    for GpioteChannel<D, P>
{
    fn mount<M>(&mut self, _: Address<Self>, broker: Broker<M>)
    where
        M: Device,
    {
        // self.broker.replace(broker);
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub struct PinEvent(pub PinState);

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum PinState {
    High,
    Low,
}

pub enum Edge {
    Rising,
    Falling,
    Both,
}

impl<D: Device + EventHandler<GpioteEvent>> Gpiote<D> {
    pub fn new(gpiote: hal::pac::GPIOTE) -> Self {
        let gpiote = hal::gpiote::Gpiote::new(gpiote);
        Self {
            gpiote,
            broker: None,
        }
    }

    pub fn configure_channel<P: InputPin + GpioteInputPin>(
        &self,
        channel: Channel,
        pin: P,
        edge: Edge,
    ) -> GpioteChannel<D, P>
    where
        D: EventHandler<PinEvent>,
    {
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

impl<D: Device + EventHandler<PinEvent>, P: InputPin + GpioteInputPin> GpioteChannel<D, P> {
    pub fn new(channel: Channel, pin: P, edge: Edge) -> GpioteChannel<D, P> {
        GpioteChannel {
            channel,
            pin,
            edge,
            broker: None,
        }
    }
}

impl<D: Device + EventHandler<PinEvent>, P: InputPin + GpioteInputPin> Sink<GpioteEvent>
    for GpioteChannel<D, P>
{
    fn notify(&self, event: GpioteEvent) {
        match event {
            GpioteEvent(c) if c == self.channel => {
                log::info!("Channel {:?} notified!", self.channel);
                if let Some(broker) = &self.broker {
                    if self.pin.is_high().ok().unwrap() {
                        broker.publish::<Self, PinEvent>(PinEvent(PinState::High));
                    } else {
                        broker.publish::<Self, PinEvent>(PinEvent(PinState::Low));
                    }
                }
            }
            _ => {}
        }
    }
}

impl<D: Device + EventHandler<GpioteEvent>> Interrupt for Gpiote<D> {
    fn on_interrupt(&mut self) {
        if let Some(broker) = &self.broker {
            if self.gpiote.channel0().is_event_triggered() {
                broker.publish::<Self, GpioteEvent>(GpioteEvent(Channel::Channel0));
            }

            if self.gpiote.channel1().is_event_triggered() {
                broker.publish::<Self, GpioteEvent>(GpioteEvent(Channel::Channel1));
            }

            if self.gpiote.channel2().is_event_triggered() {
                broker.publish::<Self, GpioteEvent>(GpioteEvent(Channel::Channel2));
            }

            if self.gpiote.channel3().is_event_triggered() {
                broker.publish::<Self, GpioteEvent>(GpioteEvent(Channel::Channel3));
            }
        }
        self.gpiote.reset_events();
    }
}

impl<D: Device + EventHandler<GpioteEvent>> Actor for Gpiote<D> {
    fn mount<MD: Device>(&mut self, _: Address<Self>, broker: Broker<MD>) {
        // self.broker.replace(broker);
    }
}

impl<D: Device + EventHandler<GpioteEvent>> NotificationHandler<Lifecycle> for Gpiote<D> {
    fn on_notification(&'static mut self, _: Lifecycle) -> Completion {
        Completion::immediate()
    }
}

impl<D: Device + EventHandler<PinEvent>, P: InputPin + GpioteInputPin + 'static>
    NotificationHandler<Lifecycle> for GpioteChannel<D, P>
{
    fn on_notification(&'static mut self, _: Lifecycle) -> Completion {
        Completion::immediate()
    }
}

impl<D: Device + EventHandler<PinEvent>, P: InputPin + GpioteInputPin + 'static> Publisher<PinEvent>
    for GpioteChannel<D, P>
{
}

impl<D: Device + EventHandler<GpioteEvent>> Publisher<GpioteEvent> for Gpiote<D> {}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum Channel {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
}

#[derive(Debug, Copy, Clone)]
pub struct GpioteEvent(pub Channel);
