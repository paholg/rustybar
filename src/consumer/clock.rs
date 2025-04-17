use iced::{widget::text, Color, Element};
use jiff::Zoned;
use serde::Deserialize;

use crate::{
    config::de_color,
    producer::{tick::TickProducer, ProducerMap},
    ConsumerEnum, Message, ProducerEnum,
};

use super::{Consumer, RegisterConsumer};

#[derive(Deserialize)]
pub struct ClockConfig {
    pub format: String,
    #[serde(deserialize_with = "de_color")]
    pub color: Color,
}

impl RegisterConsumer for ClockConfig {
    fn register(self, producers: &mut ProducerMap) -> ConsumerEnum {
        producers.register(ProducerEnum::TickProducer(TickProducer::default()));
        ClockConsumer {
            config: self,
            time: Zoned::now(),
        }
        .into()
    }
}

pub struct ClockConsumer {
    config: ClockConfig,
    time: Zoned,
}

impl Consumer for ClockConsumer {
    fn handle(&mut self, message: &Message) {
        if let Message::Tick(msg) = message {
            self.time = msg.time.clone();
        }
    }

    fn render(&self) -> Element<Message> {
        let now = self.time.strftime(&self.config.format).to_string();
        text(now).color(self.config.color).into()
    }
}
