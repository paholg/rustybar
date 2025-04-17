use enum_dispatch::enum_dispatch;
use iced::Element;

use crate::{producer::ProducerMap, ConsumerEnum, Message};

pub mod battery;
pub mod clock;
pub mod cpu;
pub mod memory;
pub mod network;
pub mod temp;

#[enum_dispatch(ConsumerEnum)]
pub trait Consumer {
    fn handle(&mut self, message: &Message);
    fn render(&self) -> Element<Message>;
}

#[enum_dispatch(ConsumerConfig)]
pub trait RegisterConsumer {
    fn register(self, producers: &mut ProducerMap) -> ConsumerEnum;
}
