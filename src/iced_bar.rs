use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use futures::{SinkExt, Stream};
use iced::theme::Palette;
use iced::widget::{Row, container, row};
use iced::{Element, Event, Font, Length, Point, Subscription, Task, Theme, mouse, window};
use iced_layershell::actions::IcedNewPopupSettings;
use iced_layershell::build_pattern::daemon;
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption, PopupAnchor,
    PopupGravity,
};
use iced_layershell::settings::{LayerShellSettings, Settings};
use tokio::sync::watch;

use crate::APP;
use crate::consumer::{IcedMessage, tray as tray_consumer};
use crate::producer::{niri, tick, tray};

pub fn run(output: String, shutdown: watch::Receiver<bool>) -> eyre::Result<()> {
    // Leak to deal with iced's boot nonsense.
    let o = Box::new(output).leak();
    let start_mode = iced_layershell::settings::StartMode::TargetScreen(o.to_owned());

    daemon(
        move || BarInstance {
            output: o.to_owned(),
            shutdown: shutdown.clone(),
            cursor: None,
            popups: HashMap::new(),
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
    /// The last pointer position seen, and the surface it was on.
    cursor: Option<(window::Id, Point)>,
    /// Open popup surfaces, keyed by their window id.
    popups: HashMap<window::Id, Popup>,
}

enum Popup {
    TrayMenu { address: String },
    Tooltip { address: String },
}

impl Popup {
    fn is_tooltip(&self) -> bool {
        matches!(self, Popup::Tooltip { .. })
    }
}

fn namespace() -> String {
    String::from("rustybar")
}

/// Close every popup matching `which`.
fn close_popups(instance: &mut BarInstance, which: fn(&Popup) -> bool) -> Task<IcedMessage> {
    let ids: Vec<_> = instance
        .popups
        .iter()
        .filter(|(_, popup)| which(popup))
        .map(|(id, _)| *id)
        .collect();
    Task::batch(ids.into_iter().map(|id| {
        instance.popups.remove(&id);
        Task::done(IcedMessage::RemoveWindow(id))
    }))
}

fn update(instance: &mut BarInstance, message: IcedMessage) -> Task<IcedMessage> {
    match message {
        IcedMessage::Exit => iced::exit(),
        IcedMessage::Cursor { window, position } => {
            instance.cursor = Some((window, position));
            Task::none()
        }
        IcedMessage::WindowClosed(id) => {
            instance.popups.remove(&id);
            Task::none()
        }
        IcedMessage::TrayActivate { address, secondary } => {
            Task::future(tray::activate(address, secondary)).discard()
        }
        IcedMessage::TrayHover { address } => {
            let close = close_popups(instance, Popup::is_tooltip);
            let Some(item) = address.and_then(|address| tray::item(&address)) else {
                return close;
            };
            let size = tray_consumer::tooltip_size(&item);
            let x = instance.cursor.map(|(_, p)| p.x as i32).unwrap_or(0);
            let left = (x - size.0 as i32 / 2).max(0);
            let id = window::Id::unique();
            instance.popups.insert(
                id,
                Popup::Tooltip {
                    address: item.address,
                },
            );
            // A popup would take a pointer grab, so use a layer surface just
            // below the bar, centered on the pointer, that ignores input.
            let settings = NewLayerShellSettings {
                size: Some(size),
                layer: Layer::Overlay,
                anchor: Anchor::Top | Anchor::Left,
                exclusive_zone: Some(0),
                margin: Some((APP.config.height as i32, 0, 0, left)),
                keyboard_interactivity: KeyboardInteractivity::None,
                output_option: OutputOption::OutputName(instance.output.clone()),
                events_transparent: true,
                namespace: Some("rustybar-tooltip".into()),
            };
            Task::batch([
                close,
                Task::done(IcedMessage::NewLayerShell { settings, id }),
            ])
        }
        IcedMessage::TrayMenuOpen { address } => {
            let close = close_popups(instance, |_| true);
            let Some(item) = tray::item(&address) else {
                return close;
            };
            let Some(menu_path) = item.menu_path.clone() else {
                return close;
            };
            let x = instance.cursor.map(|(_, p)| p.x as i32).unwrap_or(0);
            let id = window::Id::unique();
            instance.popups.insert(
                id,
                Popup::TrayMenu {
                    address: address.clone(),
                },
            );
            // Open below the bar, at the pointer's x, growing down and right.
            let settings = IcedNewPopupSettings::at_position_on_current_surface(
                tray_consumer::menu_size(&item),
                (x, APP.config.height as i32),
            )
            .anchor(PopupAnchor::TopLeft)
            .gravity(PopupGravity::BottomRight);
            Task::batch([
                close,
                Task::future(tray::menu_about_to_show(address, menu_path)).discard(),
                Task::done(IcedMessage::NewPopUp { settings, id }),
            ])
        }
        IcedMessage::TrayMenuClick {
            popup,
            address,
            menu_path,
            item,
        } => {
            instance.popups.remove(&popup);
            Task::batch([
                Task::future(tray::menu_click(address, menu_path, item)).discard(),
                Task::done(IcedMessage::RemoveWindow(popup)),
            ])
        }
        _ => Task::none(),
    }
}

fn theme(_: &BarInstance, _: window::Id) -> Theme {
    let mut palette = Palette::DARK;
    palette.background = APP.config.background;
    Theme::custom("rustybar", palette)
}

fn subscription(instance: &BarInstance) -> Subscription<IcedMessage> {
    Subscription::batch([
        Subscription::run_with(
            WorkerSeed {
                output: instance.output.clone(),
                shutdown: instance.shutdown.clone(),
            },
            worker,
        ),
        iced::event::listen_with(|event, _, window| match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(IcedMessage::Cursor { window, position })
            }
            _ => None,
        }),
        window::close_events().map(IcedMessage::WindowClosed),
    ])
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
        let mut tray_receiver = tray::listen();
        loop {
            let stop = *shutdown.borrow_and_update();
            if stop {
                output.send(IcedMessage::Exit).await.unwrap();
                return;
            }
            tokio::select! {
                _ = tick_receiver.changed() => {},
                _ = niri_receiver.changed() => {},
                _ = tray_receiver.changed() => {},
                _ = shutdown.changed() => continue,
            }
            output.send(IcedMessage::A).await.unwrap();
        }
    })
}

fn view(instance: &BarInstance, id: window::Id) -> Element<'_, IcedMessage> {
    match instance.popups.get(&id) {
        Some(Popup::TrayMenu { address }) => match tray::item(address) {
            Some(item) => tray_consumer::menu_view(id, &item),
            None => iced::widget::Space::new().into(),
        },
        Some(Popup::Tooltip { address }) => match tray::item(address) {
            Some(item) => tray_consumer::tooltip_view(&item),
            None => iced::widget::Space::new().into(),
        },
        None => bar_view(instance),
    }
}

fn bar_view(instance: &BarInstance) -> Element<'_, IcedMessage> {
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
