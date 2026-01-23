use async_trait::async_trait;
use iced::{
    Border, Color, Element, Length,
    widget::{Column as IcedColumn, Row, container},
};
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
    pub background: Color,
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

struct Window {
    height: f64,
    focused: bool,
    urgent: bool,
}

struct FloatingWindow {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    focused: bool,
    urgent: bool,
}

struct Column {
    width: f64,
    windows: Vec<Window>,
}

#[derive(Default)]
struct Windows {
    scale_factor: f64,
    cols: Vec<Column>,
    floaters: Vec<FloatingWindow>,
}

impl Windows {
    fn new(output: &Output) -> Self {
        let mut cols = Vec::new();
        let mut floaters = Vec::new();

        for window in output.workspace_windows.iter() {
            let layout = &window.layout;
            match layout.pos_in_scrolling_layout {
                Some((col, _row)) => {
                    let window = Window {
                        height: layout.tile_size.1,
                        focused: window.is_focused,
                        urgent: window.is_urgent,
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
                None => floaters.push(FloatingWindow {
                    x: layout.window_offset_in_tile.0,
                    y: layout.window_offset_in_tile.1,
                    width: layout.tile_size.0,
                    height: layout.tile_size.1,
                    focused: window.is_focused,
                    urgent: window.is_urgent,
                }),
            }
        }

        let output_height = cols
            .iter()
            .map(|col| col.windows.iter().map(|w| w.height).sum::<f64>())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or_default();
        let scale_factor = output_height / (APP.config.height as f64);

        Windows {
            scale_factor,
            cols,
            floaters,
        }
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
            return Row::new().into();
        };
        let windows = Windows::new(output);

        if windows.cols.is_empty() {
            return Row::new().into();
        }

        let config = &self.config;
        let scale = windows.scale_factor;

        Row::with_children(windows.cols.iter().map(|col| {
            let scaled_width = (col.width / scale) as f32;

            IcedColumn::with_children(col.windows.iter().map(|win| {
                let fill = if win.urgent {
                    config.urgent
                } else if win.focused {
                    config.focused
                } else {
                    config.background
                };

                let border_color = config.border;

                container(Row::new())
                    .width(Length::Fixed(scaled_width))
                    .height(Length::Fixed((win.height / scale) as f32))
                    .style(move |_| container::Style {
                        background: Some(fill.into()),
                        border: Border {
                            color: border_color,
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        ..Default::default()
                    })
                    .into()
            }))
            .into()
        }))
        .into()
    }
}
