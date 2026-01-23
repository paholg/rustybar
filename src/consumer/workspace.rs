use async_trait::async_trait;
use iced::{
    Alignment, Color, Element, Length,
    widget::{Row, Stack, container, text},
};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    consumer::{Config, Consumer, IcedMessage},
    producer::niri,
};

#[derive(Deserialize, Serialize)]
pub struct WorkspaceConfig {
    pub focused_color: Color,
    pub active_color: Color,
    pub inactive_color: Color,
    pub windowless_color: Color,
    pub urgent_background: Color,
    pub spacing: f32,
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

        Row::with_children(output.workspaces.iter().map(|ws| {
            let fg = if ws.active_window_id.is_some() {
                self.config.inactive_color
            } else {
                self.config.windowless_color
            };

            // FIXME: Handle bg color for urgency.

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
                        .align_y(Alignment::End),
                )
                .into()
        }))
        .spacing(self.config.spacing)
        .into()
    }
}
