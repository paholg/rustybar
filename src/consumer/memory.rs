use async_trait::async_trait;
use iced::{Element, widget::text};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, IcedMessage},
    producer::tick::{self},
    util::{bytes::format_bytes, color::Colormap},
};

use super::Consumer;

#[derive(Deserialize, Serialize)]
pub struct MemoryConfig {
    pub colormap: Colormap,
}

#[typetag::serde]
impl Config for MemoryConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = tick::listen();

        Box::new(MemoryConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct MemoryConsumer {
    receiver: watch::Receiver<tick::Message>,
    config: MemoryConfig,
}

#[async_trait]
impl Consumer for MemoryConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, _: &str) -> Element<'_, IcedMessage> {
        let mem = self.receiver.borrow().memory.available;
        let t = format_bytes(mem);
        let color = self.config.colormap.map(mem as f32);
        text(t).color(color).into()
    }
}
