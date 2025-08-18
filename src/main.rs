use std::cell::RefCell;

use futures::stream::select_all;
use futures::StreamExt;
use iced::futures::Stream;
use iced::theme::Palette;
use iced::widget::{container, row, Row};
use iced::{stream, Element, Font, Length, Subscription, Task, Theme};
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::Application;
use rustybar::config::{GlobalConfig, RustybarConfig};
use rustybar::consumer::{Consumer, RegisterConsumer};
use rustybar::producer::{Producer, ProducerMap};
use rustybar::{ConsumerEnum, Message, ProducerEnum};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let config = RustybarConfig::default();

    Rustybar::run(Settings {
        id: Some("rustybar".into()),
        antialiasing: true,
        default_font: Font::MONOSPACE,
        default_text_size: config.global.font_size.into(),
        layer_settings: LayerShellSettings {
            size: Some((0, config.global.height)),
            exclusive_zone: config.global.height.try_into()?,
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            start_mode: iced_layershell::settings::StartMode::TargetScreen("DP-3".into()),
            ..Default::default()
        },
        ..Default::default()
    })?;

    Ok(())
}

struct Rustybar {
    config: GlobalConfig,
    producers: RefCell<Vec<ProducerEnum>>,

    left: Vec<ConsumerEnum>,
    center: Vec<ConsumerEnum>,
    right: Vec<ConsumerEnum>,
}

impl Application for Rustybar {
    type Message = Message;
    type Flags = RustybarConfig;
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(config: RustybarConfig) -> (Self, Task<Message>) {
        let mut producer_map = ProducerMap::default();
        let left = config
            .left
            .into_iter()
            .map(|c| c.register(&mut producer_map))
            .collect();
        let center = config
            .center
            .into_iter()
            .map(|c| c.register(&mut producer_map))
            .collect();
        let right = config
            .right
            .into_iter()
            .map(|c| c.register(&mut producer_map))
            .collect();
        let this = Self {
            config: config.global,
            producers: producer_map.into_producers().into(),
            left,
            center,
            right,
        };
        (this, Task::none())
    }

    fn namespace(&self) -> String {
        String::from("rustybar")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        for component in self
            .left
            .iter_mut()
            .chain(self.center.iter_mut())
            .chain(self.right.iter_mut())
        {
            component.handle(&message);
        }
        Task::none()
    }

    fn theme(&self) -> Self::Theme {
        let mut palette = Palette::DARK;
        palette.background = self.config.background;
        Theme::custom("rustybar".into(), palette)
    }

    fn view(&self) -> Element<Message> {
        row![
            container(
                Row::with_children(self.left.iter().map(|comp| comp.render()))
                    .spacing(self.config.spacing)
            )
            .center_y(Length::Fill)
            .align_left(Length::Fill),
            container(
                Row::with_children(self.center.iter().map(|comp| comp.render()))
                    .spacing(self.config.spacing)
            )
            .center_y(Length::Fill)
            .center_x(Length::Fill),
            container(
                Row::with_children(self.right.iter().map(|comp| comp.render()))
                    .spacing(self.config.spacing)
            )
            .center_y(Length::Fill)
            .align_right(Length::Fill),
        ]
        .spacing(self.config.spacing)
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let producers = self.producers.take();
        let stream = worker(producers);
        Subscription::run_with_id("hello", stream)
    }
}

fn worker(producers: Vec<ProducerEnum>) -> impl Stream<Item = Message> {
    let streams = producers
        .into_iter()
        .map(move |p| Box::pin(p.produce_stream()));
    let mut streams = select_all(streams);
    stream::channel(100, |mut output| async move {
        loop {
            let msg = streams.next().await.unwrap();
            output.try_send(msg).unwrap();
        }
    })
}
