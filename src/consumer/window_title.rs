use async_trait::async_trait;
use iced::{Color, Element, widget::text};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, IcedMessage},
    producer::niri,
};

use super::Consumer;

#[derive(Deserialize, Serialize)]
pub struct WindowTitleConfig {
    pub color: Color,
}

#[typetag::serde]
impl Config for WindowTitleConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = niri::listen();

        Box::new(WindowTitleConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct WindowTitleConsumer {
    receiver: watch::Receiver<niri::Message>,
    config: WindowTitleConfig,
}

#[async_trait]
impl Consumer for WindowTitleConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, output: &str) -> Element<'_, IcedMessage> {
        let msg = self.receiver.borrow();
        match msg.outputs.get(output) {
            Some(output) => text(output.window.clone()).color(self.config.color).into(),
            None => text("--- MISSING ---").color(self.config.color).into(),
        }
    }
}
