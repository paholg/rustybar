use async_trait::async_trait;
use iced::{
    Color, Element,
    widget::{Row, text},
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
            let fg = if ws.is_focused {
                self.config.focused_color
            } else if ws.is_active {
                self.config.active_color
            } else if ws.active_window_id.is_some() {
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
            text(label).color(fg).into()
        }))
        .spacing(self.config.spacing)
        .into()
    }
}
