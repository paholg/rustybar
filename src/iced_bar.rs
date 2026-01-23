use futures::{SinkExt, Stream};
use iced::theme::Palette;
use iced::widget::{Row, container, row};
use iced::{Element, Font, Length, Subscription, Task, Theme};
use iced_layershell::application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};

use crate::APP;
use crate::consumer::IcedMessage;
use crate::producer::{niri, tick};

pub fn run(output: String) -> eyre::Result<()> {
    // Leak to deal with iced's boot nonsense.
    let o = Box::new(output).leak();
    let start_mode = iced_layershell::settings::StartMode::TargetScreen(o.to_owned());

    application(
        || BarInstance {
            output: o.to_owned(),
        },
        namespace,
        update,
        view,
    )
    .theme(theme)
    .subscription(subscription)
    .settings(Settings {
        id: Some("rustybar".into()),
        antialiasing: true,
        default_font: Font::MONOSPACE,
        default_text_size: APP.config.font_size.into(),
        layer_settings: LayerShellSettings {
            size: Some((0, APP.config.height)),
            exclusive_zone: APP.config.height.try_into()?,
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            start_mode,
            ..Default::default()
        },
        ..Default::default()
    })
    .run()?;

    Ok(())
}

struct BarInstance {
    output: String,
}

fn namespace() -> String {
    String::from("rustybar")
}

fn update(_: &mut BarInstance, _: IcedMessage) -> Task<IcedMessage> {
    Task::none()
}

fn theme(_: &BarInstance) -> Theme {
    let mut palette = Palette::DARK;
    palette.background = APP.config.background;
    Theme::custom("rustybar", palette)
}

fn subscription(_: &BarInstance) -> Subscription<IcedMessage> {
    Subscription::run(worker)
}

fn worker() -> impl Stream<Item = IcedMessage> {
    iced::stream::channel(1, async |mut output| {
        let mut tick_receiver = tick::listen();
        let mut niri_receiver = niri::listen();
        loop {
            tokio::select! {
                _ = tick_receiver.changed() => {},
                _ = niri_receiver.changed() => {},
            }
            output.send(IcedMessage::A).await.unwrap();
        }
    })
}

fn view(instance: &BarInstance) -> Element<'_, IcedMessage> {
    row![
        container(
            Row::with_children(APP.left.iter().map(|comp| comp.render(&instance.output)))
                .spacing(APP.config.spacing)
        )
        .center_y(Length::Fill)
        .align_left(Length::Fill),
        container(
            Row::with_children(APP.center.iter().map(|comp| comp.render(&instance.output)))
                .spacing(APP.config.spacing)
        )
        .center_y(Length::Fill),
        container(
            Row::with_children(APP.right.iter().map(|comp| comp.render(&instance.output)))
                .spacing(APP.config.spacing)
        )
        .center_y(Length::Fill)
        .align_right(Length::Fill),
    ]
    .spacing(APP.config.spacing)
    .into()
}
