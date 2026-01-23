use async_trait::async_trait;
use iced::{Color, Element, Length, widget::Svg};
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
    app_id: Option<String>,
    focused: bool,
    urgent: bool,
}

#[allow(dead_code)]
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
    #[allow(dead_code)]
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
                    x: layout.tile_pos_in_workspace_view.unwrap().0,
                    y: layout.tile_pos_in_workspace_view.unwrap().1,
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

fn color_to_svg(c: Color) -> String {
    format!(
        "rgb({},{},{})",
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8
    )
}

#[async_trait]
impl Consumer for WindowDiagramConsumer {
    async fn consume(&mut self) {
        self.receiver.changed().await.unwrap();
    }

    fn render(&self, output_name: &str) -> Element<'_, IcedMessage> {
        let msg = self.receiver.borrow();
        let Some(output) = msg.outputs.get(output_name) else {
            return Svg::new(iced::widget::svg::Handle::from_memory(
                b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>".to_vec(),
            ))
            .width(Length::Shrink)
            .height(Length::Fill)
            .into();
        };
        let windows = Windows::new(output);

        if windows.cols.is_empty() {
            return Svg::new(iced::widget::svg::Handle::from_memory(
                b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>".to_vec(),
            ))
            .width(Length::Shrink)
            .height(Length::Fill)
            .into();
        }

        let total_width: f64 = windows.cols.iter().map(|c| c.width).sum();
        let scale = windows.scale_factor;
        let scaled_width = total_width / scale;
        let scaled_height = APP.config.height as f64;

        let config = &self.config;
        let border_color = color_to_svg(config.border);

        // Build SVG content (add 1 to viewBox to account for stroke width)
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">"#,
            scaled_width + 1.0,
            scaled_height + 1.0
        );

        // Draw tiled columns
        let mut x = 0.0;
        for col in &windows.cols {
            let w = col.width / scale;
            let mut y = 0.0;

            for win in &col.windows {
                let h = win.height / scale;
                let fill = if win.urgent {
                    color_to_svg(config.urgent)
                } else if win.focused {
                    color_to_svg(config.focused)
                } else {
                    color_to_svg(config.background)
                };

                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="2"/>"#,
                    x, y, w, h, fill, border_color
                ));

                y += h;
            }
            x += w;
        }

        // TODO: Draw floating windows on top once niri provides tile_pos_in_workspace_view for tiled windows

        svg.push_str("</svg>");

        Svg::new(iced::widget::svg::Handle::from_memory(svg.into_bytes()))
            .width(Length::Fixed((scaled_width + 1.0) as f32))
            .height(Length::Fill)
            .into()
    }
}
