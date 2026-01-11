use async_trait::async_trait;
use iced::{
    Color, Element, Length, Theme,
    alignment::Vertical,
    border::Radius,
    widget::{ProgressBar, row, text},
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
pub struct BatteryConfig {
    pub width: f32,
    pub height: f32,
    pub spacing: f32,
    pub colormap: Colormap,
    pub colors: BatteryColors,
}

#[typetag::serde]
impl Config for BatteryConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = tick::listen();

        Box::new(BatteryConsumer {
            receiver,
            config: *self,
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct BatteryColors {
    pub charge: Color,
    pub discharge: Color,
    pub unknown: Color,
}

pub struct BatteryConsumer {
    receiver: watch::Receiver<tick::Message>,
    config: BatteryConfig,
}

impl BatteryConsumer {
    fn bar(&self, charge: f32) -> ProgressBar<'_, Theme> {
        let color = self.config.colormap.map(charge);
        iced::widget::progress_bar(0.0..=1.0, charge)
            .length(Length::Fixed(self.config.width))
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
impl Consumer for BatteryConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, _: &str) -> Element<'_, IcedMessage> {
        let Some(battery) = &self.receiver.borrow().battery else {
            return row![].into();
        };

        let colors = &self.config.colors;
        let text = match battery.state {
            starship_battery::State::Unknown => text('*').color(colors.unknown),
            starship_battery::State::Charging => text('+').color(colors.charge),
            starship_battery::State::Discharging => text('-').color(colors.discharge),
            starship_battery::State::Empty => text('!').color(colors.unknown),
            starship_battery::State::Full => text(' '),
        };

        row![self.bar(battery.charge), text]
            .align_y(Vertical::Center)
            .spacing(self.config.spacing)
            .into()
    }
}
