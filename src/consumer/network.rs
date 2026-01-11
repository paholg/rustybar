use async_trait::async_trait;
use iced::{
    Element,
    alignment::Vertical,
    widget::{Text, row, text},
};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, IcedMessage},
    producer::tick::{self},
    util::{bytes::format_bytes, color::Colormap},
};

use super::Consumer;

#[derive(Deserialize, Serialize)]
pub struct NetworkConfig {
    pub colormap: Colormap,
    pub spacing: f32,
}

#[typetag::serde]
impl Config for NetworkConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = tick::listen();

        Box::new(NetworkConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct NetworkConsumer {
    receiver: watch::Receiver<tick::Message>,
    config: NetworkConfig,
}

impl NetworkConsumer {
    fn text(&self, value: u64) -> Text<'_> {
        text(format_bytes(value)).color(self.config.colormap.map(value as f32))
    }
}

#[async_trait]
impl Consumer for NetworkConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, _: &str) -> Element<'_, IcedMessage> {
        let network = &self.receiver.borrow().network;
        row![
            self.text(network.bytes_received),
            self.text(network.bytes_transmitted),
        ]
        .align_y(Vertical::Center)
        .spacing(self.config.spacing)
        .into()
    }
}
