use async_trait::async_trait;
use iced::{Color, Element, widget::row};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

use crate::{
    APP,
    consumer::{Config, IcedMessage},
    producer::niri::{self, Output},
};

use super::Consumer;

#[derive(Deserialize, Serialize)]
pub struct WindowDiagramConfig {
    pub border: Color,
    pub focused: Color,
    pub active: Color,
    pub urgent: Color,
    pub visible: Color,
}

#[typetag::serde]
impl Config for WindowDiagramConfig {
    fn into_consumer(self: Box<Self>) -> Box<dyn Consumer> {
        let receiver = niri::listen();

        Box::new(WindowDiagramConsumer {
            receiver,
            config: *self,
        })
    }
}

pub struct WindowDiagramConsumer {
    receiver: watch::Receiver<niri::Message>,
    config: WindowDiagramConfig,
}

struct WindowInfo {
    height: f64,
    focused: bool,
    urgent: bool,
    floating: bool,
}

struct Column {
    width: f64,
    windows: Vec<WindowInfo>,
}

#[derive(Default)]
struct Windows {
    scale_factor: f64,
    cols: Vec<Column>,
}

impl Windows {
    fn new(output: &Output) -> Self {
        let Some(last_window) = output.workspace_windows.last() else {
            return Windows::default();
        };
        let mut cols = Vec::with_capacity(last_window.layout.pos_in_scrolling_layout.unwrap().0);

        for window in output.workspace_windows.iter() {
            let layout = &window.layout;
            let (col, _row) = layout.pos_in_scrolling_layout.unwrap();

            let window = WindowInfo {
                height: layout.tile_size.1,
                focused: window.is_focused,
                urgent: window.is_urgent,
                floating: window.is_urgent,
            };

            if col > cols.len() {
                cols.push(Column {
                    width: layout.tile_size.0,
                    windows: vec![window],
                })
            } else {
                cols.last_mut().unwrap().windows.push(window);
            }
        }

        let output_height = cols
            .iter()
            .map(|col| col.windows.iter().map(|w| w.height).sum::<f64>())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let scale_factor = output_height / (APP.config.height as f64);

        Windows { scale_factor, cols }
    }
}

#[async_trait]
impl Consumer for WindowDiagramConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, output_name: &str) -> Element<'_, IcedMessage> {
        let msg = self.receiver.borrow();
        let Some(output) = msg.outputs.get(output_name) else {
            return row![].into();
        };
        let windows = Windows::new(output);

        row![].into()
    }
}
