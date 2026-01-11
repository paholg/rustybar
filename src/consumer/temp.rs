use async_trait::async_trait;
use iced::{Element, widget::text};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, IcedMessage},
    producer::tick::{self},
    util::color::Colormap,
};

use super::Consumer;

#[derive(Deserialize, Serialize)]
pub struct TempConfig {
    pub colormap: Colormap,
}

#[typetag::serde]
impl Config for TempConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = tick::listen();

        Box::new(TempConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct TempConsumer {
    receiver: watch::Receiver<tick::Message>,
    config: TempConfig,
}

#[async_trait]
impl Consumer for TempConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, _: &str) -> Element<'_, IcedMessage> {
        let max = self.receiver.borrow().temp.max;
        let t = format!("{max:3.0} °C");
        let color = self.config.colormap.map(max);
        text(t).color(color).into()
    }
}
