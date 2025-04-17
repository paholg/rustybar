use iced::{widget::text, Element};
use serde::Deserialize;

use crate::{
    producer::{
        tick::{Memory, TickProducer},
        ProducerMap,
    },
    util::{bytes::format_bytes, color::Colormap},
    ConsumerEnum, Message, ProducerEnum,
};

use super::{Consumer, RegisterConsumer};

#[derive(Deserialize)]
pub struct MemoryConfig {
    pub colormap: Colormap,
}

impl RegisterConsumer for MemoryConfig {
    fn register(self, producers: &mut ProducerMap) -> ConsumerEnum {
        producers.register(ProducerEnum::TickProducer(TickProducer::default()));
        MemoryConsumer {
            config: self,
            mem: Memory::default(),
        }
        .into()
    }
}

pub struct MemoryConsumer {
    config: MemoryConfig,
    mem: Memory,
}

impl Consumer for MemoryConsumer {
    fn handle(&mut self, message: &Message) {
        if let Message::Tick(msg) = message {
            self.mem = msg.memory.clone();
        }
    }

    fn render(&self) -> Element<Message> {
        let mem = self.mem.available;
        let t = format_bytes(mem);
        let color = self.config.colormap.map(mem as f32);
        text(t).color(color).into()
    }
}
