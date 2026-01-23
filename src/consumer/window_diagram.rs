use std::{fs, path::PathBuf};

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
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
                        app_id: window.app_id.clone(),
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

fn find_icon(app_id: &str) -> Option<PathBuf> {
    let extensions = ["svg", "png"];

    // Standard icon locations
    let icon_dirs = [
        "/run/current-system/sw/share/icons/hicolor/scalable/apps",
        "/run/current-system/sw/share/icons/hicolor/256x256/apps",
        "/run/current-system/sw/share/icons/hicolor/128x128/apps",
        "/run/current-system/sw/share/icons/hicolor/64x64/apps",
        "/run/current-system/sw/share/icons/hicolor/48x48/apps",
        "/run/current-system/sw/share/pixmaps",
    ];

    for dir in &icon_dirs {
        for ext in &extensions {
            let path = PathBuf::from(dir).join(format!("{}.{}", app_id, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }

    // On NixOS, try to find icon in the app's nix store path
    if let Ok(bin_path) = std::process::Command::new("which").arg(app_id).output() {
        if bin_path.status.success() {
            let bin_path = String::from_utf8_lossy(&bin_path.stdout).trim().to_string();
            if let Ok(resolved) = fs::canonicalize(&bin_path) {
                // Go up to the nix store package root (2 levels up from bin/)
                if let Some(store_path) = resolved.ancestors().nth(2) {
                    // Search for icons in share/icons and lib/*/logo
                    let search_dirs = [
                        store_path.join("share/icons/hicolor/scalable/apps"),
                        store_path.join("share/icons/hicolor/256x256/apps"),
                        store_path.join("share/icons/hicolor/128x128/apps"),
                        store_path.join("share/icons/hicolor/64x64/apps"),
                        store_path.join("share/icons/hicolor/48x48/apps"),
                        store_path.join("share/pixmaps"),
                    ];

                    for dir in &search_dirs {
                        for ext in &extensions {
                            let path = dir.join(format!("{}.{}", app_id, ext));
                            if path.exists() {
                                return Some(path);
                            }
                        }
                    }

                    // Also try lib/{app}/logo/{app}.png pattern (used by kitty)
                    for ext in &extensions {
                        let path =
                            store_path.join(format!("lib/{}/logo/{}.{}", app_id, app_id, ext));
                        if path.exists() {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    None
}

fn load_icon_data_url(path: &PathBuf) -> Option<String> {
    let data = fs::read(path).ok()?;
    let ext = path.extension()?.to_str()?;

    let mime = match ext {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => return None,
    };

    Some(format!("data:{};base64,{}", mime, BASE64.encode(&data)))
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
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="1.5"/>"#,
                    x, y, w, h, fill, border_color
                ));

                // Draw icon if found
                if let Some(app_id) = &win.app_id {
                    if let Some(icon_path) = find_icon(app_id) {
                        if let Some(data_url) = load_icon_data_url(&icon_path) {
                            let icon_size = h.min(w) * 0.8;
                            let icon_x = x + (w - icon_size) / 2.0;
                            let icon_y = y + (h - icon_size) / 2.0;
                            svg.push_str(&format!(
                                r#"<image x="{}" y="{}" width="{}" height="{}" href="{}"/>"#,
                                icon_x, icon_y, icon_size, icon_size, data_url
                            ));
                        }
                    }
                }

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
