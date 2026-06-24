use async_trait::async_trait;
use iced::{
    Alignment, Color, Element, Length, Padding,
    widget::{Stack, container, text},
};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, Consumer, IcedMessage},
    producer::niri,
    util::overflow_row::OverflowRow,
};

#[derive(Deserialize, Serialize)]
pub struct WorkspaceConfig {
    pub focused_color: Color,
    pub active_color: Color,
    pub inactive_color: Color,
    pub windowless_color: Color,
    pub urgent_color: Color,
    pub spacing: f32,
    /// Max width as a fraction of the bar region's available width
    /// (1.0 = the full region). When exceeded, the workspace list is clipped
    /// and scrolled to keep the active workspace centered.
    #[serde(default = "default_max_width")]
    pub max_width: f32,
}

fn default_max_width() -> f32 {
    1.0
}

#[typetag::serde]
impl Config for WorkspaceConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = niri::listen();

        Box::new(WorkspaceConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct WorkspaceConsumer {
    receiver: watch::Receiver<niri::Message>,
    config: WorkspaceConfig,
}

#[async_trait]
impl Consumer for WorkspaceConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, output: &str) -> Element<'_, IcedMessage> {
        let msg = self.receiver.borrow();
        let Some(output) = msg.outputs.get(output) else {
            return text("------ MISSSING -----").into();
        };

        let active = output.workspaces.iter().position(|ws| ws.is_active);
        let urgent = output
            .workspaces
            .iter()
            .enumerate()
            .filter(|(_, ws)| ws.is_urgent)
            .map(|(i, _)| i)
            .collect();

        let separator = || text("…").color(self.config.windowless_color).into();

        let workspaces = output.workspaces.iter().map(|ws| {
            let fg = if ws.is_urgent {
                self.config.urgent_color
            } else if ws.active_window_id.is_some() {
                self.config.inactive_color
            } else {
                self.config.windowless_color
            };

            let label = ws
                .name
                .as_deref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| ws.idx.to_string());

            let underline_color = if ws.is_focused {
                self.config.focused_color
            } else if ws.is_active {
                self.config.active_color
            } else {
                Color::TRANSPARENT
            };

            let underline = container(text(""))
                .height(3)
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(underline_color.into()),
                    ..Default::default()
                });

            Stack::new()
                .push(text(label).color(fg))
                .push(
                    container(underline)
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .align_y(Alignment::End)
                        // Lift the underline 1px off the bottom so its
                        // anti-aliased fringe stays inside this row's layer
                        // bounds. iced clamps incremental-repaint damage to the
                        // layer bounds, so a fringe bleeding past the edge never
                        // gets cleared and survives as a faint ghost in one of
                        // the rotating back-buffers.
                        .padding(Padding::ZERO.bottom(1.0)),
                )
                .into()
        });

        OverflowRow::new(
            workspaces,
            [separator(), separator()],
            active,
            urgent,
            self.config.max_width,
            self.config.spacing,
        )
        .into()
    }
}
