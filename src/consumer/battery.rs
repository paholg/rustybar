use iced::{
    alignment::Vertical,
    border::Radius,
    widget::{row, text, ProgressBar},
    Color, Element, Length, Theme,
};
use serde::Deserialize;

use crate::{
    config::de_color,
    producer::{
        tick::{Battery, TickProducer},
        ProducerMap,
    },
    util::color::Colormap,
    ConsumerEnum, Message, ProducerEnum,
};

use super::{Consumer, RegisterConsumer};

#[derive(Deserialize)]
pub struct BatteryConfig {
    pub width: f32,
    pub height: f32,
    pub spacing: f32,
    pub colormap: Colormap,
    pub colors: BatteryColors,
}

#[derive(Deserialize)]
pub struct BatteryColors {
    #[serde(deserialize_with = "de_color")]
    pub charge: Color,
    #[serde(deserialize_with = "de_color")]
    pub discharge: Color,
    #[serde(deserialize_with = "de_color")]
    pub unknown: Color,
}

impl RegisterConsumer for BatteryConfig {
    fn register(self, producers: &mut ProducerMap) -> ConsumerEnum {
        producers.register(ProducerEnum::TickProducer(TickProducer::default()));
        BatteryConsumer {
            config: self,
            battery: Battery::default(),
        }
        .into()
    }
}

pub struct BatteryConsumer {
    config: BatteryConfig,
    battery: Battery,
}

impl BatteryConsumer {
    fn bar(&self) -> ProgressBar<'_, Theme> {
        let color = self.config.colormap.map(self.battery.charge);
        iced::widget::progress_bar(0.0..=1.0, self.battery.charge)
            .width(Length::Fixed(self.config.width))
            .height(Length::Fixed(self.config.height))
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

impl Consumer for BatteryConsumer {
    fn handle(&mut self, message: &Message) {
        if let Message::Tick(msg) = message {
            self.battery = msg.battery.clone();
        }
    }

    fn render(&self) -> Element<Message> {
        let colors = &self.config.colors;
        let text = match self.battery.state {
            starship_battery::State::Unknown => text('*').color(colors.unknown),
            starship_battery::State::Charging => text('+').color(colors.charge),
            starship_battery::State::Discharging => text('-').color(colors.discharge),
            starship_battery::State::Empty => text('!').color(colors.unknown),
            starship_battery::State::Full => text(' '),
        };

        row![self.bar(), text]
            .align_y(Vertical::Center)
            .spacing(self.config.spacing)
            .into()
    }
}
