use async_trait::async_trait;
use iced::{
    Color, Element,
    widget::{row, text},
};
use iced_core::text::Wrapping;
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
        let txt = match msg.outputs.get(output) {
            Some(output) => output.window.clone(),
            None => "--- MISSING ---".into(),
        };
        row![text(txt).wrapping(Wrapping::None).color(self.config.color)]
            .clip(true)
            .into()
    }
}
