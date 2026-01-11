use async_trait::async_trait;
use iced::{Color, Element, widget::text};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, IcedMessage},
    producer::tick,
};

use super::Consumer;

#[derive(Deserialize, Serialize)]
pub struct ClockConfig {
    pub format: String,
    pub color: Color,
}

#[typetag::serde]
impl Config for ClockConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = tick::listen();

        Box::new(ClockConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct ClockConsumer {
    receiver: watch::Receiver<tick::Message>,
    config: ClockConfig,
}

#[async_trait]
impl Consumer for ClockConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, _: &str) -> Element<'_, IcedMessage> {
        let time = &self.receiver.borrow().time;
        let now = time.strftime(&self.config.format).to_string();
        text(now).color(self.config.color).into()
    }
}
