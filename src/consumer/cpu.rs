use iced::{
    alignment::Vertical,
    border::Radius,
    widget::{row, ProgressBar},
    Element, Length, Theme,
};
use serde::Deserialize;

use crate::{
    producer::{
        tick::{Cpu, TickProducer},
        ProducerMap,
    },
    util::color::Colormap,
    ConsumerEnum, Message, ProducerEnum,
};

use super::{Consumer, RegisterConsumer};

#[derive(Deserialize)]
pub struct CpuConfig {
    pub min_max_width: f32,
    pub avg_width: f32,
    pub colormap: Colormap,
    pub spacing: f32,
    pub height: f32,
}

impl RegisterConsumer for CpuConfig {
    fn register(self, producers: &mut ProducerMap) -> ConsumerEnum {
        producers.register(ProducerEnum::TickProducer(TickProducer::default()));
        CpuConsumer {
            config: self,
            cpu: Cpu::default(),
        }
        .into()
    }
}

pub struct CpuConsumer {
    config: CpuConfig,
    cpu: Cpu,
}

impl CpuConsumer {
    fn bar(&self, value: f32, width: f32) -> ProgressBar<'_, Theme> {
        let color = self.config.colormap.map(value);
        iced::widget::progress_bar(0.0..=1.0, value)
            .width(Length::Fixed(width))
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

impl Consumer for CpuConsumer {
    fn handle(&mut self, message: &Message) {
        if let Message::Tick(msg) = message {
            self.cpu = msg.cpu.clone();
        }
    }

    fn render(&self) -> Element<Message> {
        row![
            self.bar(self.cpu.min, self.config.min_max_width),
            self.bar(self.cpu.avg, self.config.avg_width),
            self.bar(self.cpu.max, self.config.min_max_width),
        ]
        .align_y(Vertical::Center)
        .spacing(self.config.spacing)
        .into()
    }
}
