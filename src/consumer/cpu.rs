use async_trait::async_trait;
use iced::{
    Element, Length, Theme,
    alignment::Vertical,
    border::Radius,
    widget::{ProgressBar, row},
};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, IcedMessage},
    producer::tick::{self},
    util::color::Colormap,
};

use super::Consumer;

#[derive(Deserialize, Serialize)]
pub struct CpuConfig {
    pub min_max_width: f32,
    pub avg_width: f32,
    pub colormap: Colormap,
    pub spacing: f32,
    pub height: f32,
}

#[typetag::serde]
impl Config for CpuConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = tick::listen();

        Box::new(CpuConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct CpuConsumer {
    receiver: watch::Receiver<tick::Message>,
    config: CpuConfig,
}

impl CpuConsumer {
    fn bar(&self, value: f32, width: f32) -> ProgressBar<'_, Theme> {
        let color = self.config.colormap.map(value);
        iced::widget::progress_bar(0.0..=1.0, value)
            .length(Length::Fixed(width))
            .girth(Length::Fixed(self.config.height))
            .style(move |theme: &Theme| iced::widget::progress_bar::Style {
                bar: color.into(),
                border: iced::Border {
                    color,
                    width: 1.0,
                    radius: Radius::new(0.0),
                },
                background: theme.palette().background.into(),
            })
    }
}

#[async_trait]
impl Consumer for CpuConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, _: &str) -> Element<'_, IcedMessage> {
        let cpu = &self.receiver.borrow().cpu;
        row![
            self.bar(cpu.min, self.config.min_max_width),
            self.bar(cpu.avg, self.config.avg_width),
            self.bar(cpu.max, self.config.min_max_width),
        ]
        .align_y(Vertical::Center)
        .spacing(self.config.spacing)
        .into()
    }
}
