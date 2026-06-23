use std::hash::{Hash, Hasher};

use futures::{SinkExt, Stream};
use iced::theme::Palette;
use iced::widget::{Row, container, row};
use iced::{Element, Font, Length, Subscription, Task, Theme};
use iced_layershell::application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};
use tokio::sync::watch;

use crate::APP;
use crate::consumer::IcedMessage;
use crate::producer::{niri, tick};

pub fn run(output: String, shutdown: watch::Receiver<bool>) -> eyre::Result<()> {
    // Leak to deal with iced's boot nonsense.
    let o = Box::new(output).leak();
    let start_mode = iced_layershell::settings::StartMode::TargetScreen(o.to_owned());

    application(
        move || BarInstance {
            output: o.to_owned(),
            shutdown: shutdown.clone(),
        },
        namespace,
        update,
        view,
    )
    .theme(theme)
    // Set the style directly: iced_layershell computes the initial style from
    // the *default* (light) theme and only applies our theme after the first
    // message, which flashes the bar white on every surface creation.
    .style(|_, theme| iced::theme::Style {
        background_color: APP.config.background,
        text_color: theme.palette().text,
    })
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
    shutdown: watch::Receiver<bool>,
}

fn namespace() -> String {
    String::from("rustybar")
}

fn update(_: &mut BarInstance, message: IcedMessage) -> Task<IcedMessage> {
    match message {
        IcedMessage::Exit => iced::exit(),
        _ => Task::none(),
    }
}

fn theme(_: &BarInstance) -> Theme {
    let mut palette = Palette::DARK;
    palette.background = APP.config.background;
    Theme::custom("rustybar", palette)
}

fn subscription(instance: &BarInstance) -> Subscription<IcedMessage> {
    Subscription::run_with(
        WorkerSeed {
            output: instance.output.clone(),
            shutdown: instance.shutdown.clone(),
        },
        worker,
    )
}

/// Carries the shutdown receiver into the worker subscription. Identified by
/// output name only, since the receiver isn't `Hash`.
struct WorkerSeed {
    output: String,
    shutdown: watch::Receiver<bool>,
}

impl Hash for WorkerSeed {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.output.hash(state);
    }
}

fn worker(seed: &WorkerSeed) -> impl Stream<Item = IcedMessage> + use<> {
    let mut shutdown = seed.shutdown.clone();
    iced::stream::channel(1, async move |mut output| {
        let mut tick_receiver = tick::listen();
        let mut niri_receiver = niri::listen();
        loop {
            let stop = *shutdown.borrow_and_update();
            if stop {
                output.send(IcedMessage::Exit).await.unwrap();
                return;
            }
            tokio::select! {
                _ = tick_receiver.changed() => {},
                _ = niri_receiver.changed() => {},
                _ = shutdown.changed() => continue,
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
