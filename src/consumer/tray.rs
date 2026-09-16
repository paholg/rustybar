use async_trait::async_trait;
use iced::{
    Background, Border, ContentFit, Element, Length, Padding, Theme,
    widget::{Column, Row, button, container, image, mouse_area, rule, svg, text},
    window,
};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    APP,
    consumer::{Config, IcedMessage},
    producer::tray::{self, Icon, Item, MenuEntry},
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
            .on_middle_press(activate(true))
            .on_right_press(IcedMessage::TrayMenuOpen {
                address: item.address.clone(),
            })
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

// --- Popup menu ------------------------------------------------------------

const MENU_ROW_HEIGHT: f32 = 28.0;
const MENU_SEPARATOR_HEIGHT: f32 = 9.0;
const MENU_PADDING: f32 = 4.0;
const MENU_INDENT: f32 = 16.0;
/// Room for the check mark column, in characters.
const MENU_CHECK_CHARS: f32 = 2.0;

fn menu_rows(entries: &[MenuEntry], depth: usize) -> (f32, f32) {
    entries.iter().fold((0.0, 0.0), |(height, width), entry| {
        let (h, w) = if entry.separator {
            (MENU_SEPARATOR_HEIGHT, 0.0)
        } else {
            (
                MENU_ROW_HEIGHT,
                depth as f32 * MENU_INDENT
                    + (entry.label.chars().count() as f32 + MENU_CHECK_CHARS)
                        * APP.config.font_size
                        * 0.62,
            )
        };
        let (ch, cw) = menu_rows(&entry.children, depth + 1);
        (height + h + ch, width.max(w).max(cw))
    })
}

/// Popup surface size for `item`'s menu. Popups can't be resized to fit
/// their content after creation, so this estimates from the entries.
pub fn menu_size(item: &Item) -> (u32, u32) {
    let (height, width) = menu_rows(&item.menu, 0);
    let pad = 2.0 * MENU_PADDING;
    ((width + pad + 24.0) as u32, (height + pad) as u32)
}

fn menu_entry(
    popup: window::Id,
    item: &Item,
    entry: &MenuEntry,
    depth: usize,
) -> Element<'static, IcedMessage> {
    let indent = depth as f32 * MENU_INDENT;
    if entry.separator {
        return container(rule::horizontal(1))
            .padding(Padding {
                top: MENU_SEPARATOR_HEIGHT / 2.0 - 0.5,
                bottom: MENU_SEPARATOR_HEIGHT / 2.0 - 0.5,
                left: indent,
                right: 0.0,
            })
            .into();
    }

    let mark = match entry.checked {
        Some(true) => "✓ ",
        Some(false) => "  ",
        None => "",
    };
    let label = text(format!("{mark}{}", entry.label));
    let on_press =
        (entry.enabled && entry.children.is_empty()).then(|| IcedMessage::TrayMenuClick {
            popup,
            address: item.address.clone(),
            menu_path: item.menu_path.clone().unwrap_or_default(),
            item: entry.id,
        });
    let enabled = entry.enabled;

    let row = button(label)
        .on_press_maybe(on_press)
        .width(Length::Fill)
        .height(Length::Fixed(MENU_ROW_HEIGHT))
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: indent + 6.0,
            right: 6.0,
        })
        .style(move |theme: &Theme, status| {
            let palette = theme.palette();
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: hovered.then_some(Background::Color(palette.primary)),
                text_color: if enabled {
                    palette.text
                } else {
                    palette.text.scale_alpha(0.4)
                },
                ..button::Style::default()
            }
        });

    let children = entry
        .children
        .iter()
        .map(|child| menu_entry(popup, item, child, depth + 1));
    Column::with_children(std::iter::once(row.into()).chain(children)).into()
}

/// The contents of the popup menu surface `popup` for `item`.
pub fn menu_view(popup: window::Id, item: &Item) -> Element<'static, IcedMessage> {
    let entries = item
        .menu
        .iter()
        .map(|entry| menu_entry(popup, item, entry, 0));
    container(Column::with_children(entries).width(Length::Fill))
        .padding(MENU_PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(APP.config.background)),
            border: Border {
                color: theme.palette().text.scale_alpha(0.5),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}
