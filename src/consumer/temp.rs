use iced::{widget::text, Element};
use serde::Deserialize;

use crate::{
    producer::{
        tick::{Temperature, TickProducer},
        ProducerMap,
    },
    util::color::Colormap,
    ConsumerEnum, Message, ProducerEnum,
};

use super::{Consumer, RegisterConsumer};

#[derive(Deserialize)]
pub struct TempConfig {
    pub colormap: Colormap,
}

impl RegisterConsumer for TempConfig {
    fn register(self, producers: &mut ProducerMap) -> ConsumerEnum {
        producers.register(ProducerEnum::TickProducer(TickProducer::default()));
        TempConsumer {
            config: self,
            temp: Temperature::default(),
        }
        .into()
    }
}

pub struct TempConsumer {
    config: TempConfig,
    temp: Temperature,
}

impl Consumer for TempConsumer {
    fn handle(&mut self, message: &Message) {
        if let Message::Tick(msg) = message {
            self.temp = msg.temp.clone();
        }
    }

    fn render(&self) -> Element<Message> {
        let max = self.temp.max;
        let t = format!("{max:3.0} °C");
        let color = self.config.colormap.map(max);
        text(t).color(color).into()
    }
}
