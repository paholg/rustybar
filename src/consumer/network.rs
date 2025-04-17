use iced::{
    alignment::Vertical,
    widget::{row, text, Text},
    Element,
};
use serde::Deserialize;

use crate::{
    producer::{
        tick::{Network, TickProducer},
        ProducerMap,
    },
    util::{bytes::format_bytes, color::Colormap},
    ConsumerEnum, Message, ProducerEnum,
};

use super::{Consumer, RegisterConsumer};

#[derive(Deserialize)]
pub struct NetworkConfig {
    pub colormap: Colormap,
    pub spacing: f32,
}

impl RegisterConsumer for NetworkConfig {
    fn register(self, producers: &mut ProducerMap) -> ConsumerEnum {
        producers.register(ProducerEnum::TickProducer(TickProducer::default()));
        NetworkConsumer {
            config: self,
            network: Network::default(),
        }
        .into()
    }
}

pub struct NetworkConsumer {
    config: NetworkConfig,
    network: Network,
}

impl NetworkConsumer {
    fn text(&self, value: u64) -> Text {
        text(format_bytes(value)).color(self.config.colormap.map(value as f32))
    }
}

impl Consumer for NetworkConsumer {
    fn handle(&mut self, message: &Message) {
        if let Message::Tick(msg) = message {
            self.network = msg.network.clone();
        }
    }

    fn render(&self) -> Element<Message> {
        row![
            self.text(self.network.bytes_received),
            self.text(self.network.bytes_transmitted),
        ]
        .align_y(Vertical::Center)
        .spacing(self.config.spacing)
        .into()
    }
}
