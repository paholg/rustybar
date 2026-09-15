use async_trait::async_trait;
use iced::{
    ContentFit, Element, Length,
    widget::{Row, container, image, mouse_area, svg, text},
};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, IcedMessage},
    producer::tray::{self, Icon, Item},
};

use super::Consumer;

#[derive(Deserialize, Serialize)]
pub struct TrayConfig {
    pub icon_size: f32,
    pub spacing: f32,
}

#[typetag::serde]
impl Config for TrayConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = tray::listen();

        Box::new(TrayConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct TrayConsumer {
    receiver: watch::Receiver<tray::Message>,
    config: TrayConfig,
}

impl TrayConsumer {
    fn render_item(&self, item: &Item) -> Element<'static, IcedMessage> {
        let size = Length::Fixed(self.config.icon_size);
        let icon: Element<'static, IcedMessage> = match &item.icon {
            Icon::Raster(handle) => image(handle.clone())
                .width(size)
                .height(size)
                .content_fit(ContentFit::Contain)
                .into(),
            Icon::Svg(handle) => svg(handle.clone())
                .width(size)
                .height(size)
                .content_fit(ContentFit::Contain)
                .into(),
            Icon::Missing => text(item.id.clone()).into(),
        };

        let activate = |secondary| IcedMessage::TrayActivate {
            address: item.address.clone(),
            secondary,
        };
        mouse_area(container(icon).center_y(Length::Fill))
            .on_press(activate(false))
            .on_right_press(activate(true))
            .on_middle_press(activate(true))
            .into()
    }
}

#[async_trait]
impl Consumer for TrayConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, _: &str) -> Element<'_, IcedMessage> {
        let msg = self.receiver.borrow();
        Row::with_children(msg.items.iter().map(|item| self.render_item(item)))
            .spacing(self.config.spacing)
            .into()
    }
}
