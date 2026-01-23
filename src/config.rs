use std::str::FromStr;

use iced::Color;
use serde::Deserialize;

use crate::consumer::{
    Config,
    battery::{BatteryColors, BatteryConfig},
    clock::ClockConfig,
    cpu::CpuConfig,
    memory::MemoryConfig,
    network::NetworkConfig,
    temp::TempConfig,
    window_diagram::WindowDiagramConfig,
    window_title::WindowTitleConfig,
    workspace::WorkspaceConfig,
};

#[derive(Deserialize)]
#[serde(default)]
pub struct RustybarConfig {
    pub global: GlobalConfig,
    pub left: Vec<Box<dyn Config>>,
    pub center: Vec<Box<dyn Config>>,
    pub right: Vec<Box<dyn Config>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct GlobalConfig {
    pub height: u32,
    pub background: Color,
    pub font_size: f32,
    pub spacing: f32,
    #[serde(skip)]
    pub output: Option<String>,
}

impl Default for RustybarConfig {
    fn default() -> Self {
        // Spacemacs dark colors
        let bg1 = Color::from_str("#292b2e").unwrap();
        let bg2 = Color::from_str("#1f2022").unwrap();
        let _bg4 = Color::from_str("#0a0814").unwrap();
        let aqua = Color::from_str("#2d9574").unwrap();
        let blue = Color::from_str("#4f97d7").unwrap();
        let magenta = Color::from_str("#a31db1").unwrap();
        let red = Color::from_str("#f2241f").unwrap();

        Self {
            global: GlobalConfig {
                height: 28,
                background: Color::from_str("#000000").unwrap(),
                font_size: 18.0,
                spacing: 12.0,
                output: None,
            },
            left: vec![
                Box::new(WorkspaceConfig {
                    focused_color: aqua,
                    active_color: blue,
                    inactive_color: Color::from_str("#aaaaaa").unwrap(),
                    windowless_color: Color::from_str("#666666").unwrap(),
                    urgent_color: red,
                    spacing: 12.0,
                }),
                Box::new(WindowTitleConfig { color: blue }),
            ],
            center: vec![
                Box::new(TempConfig {
                    colormap: [(40.0, aqua), (60.0, blue), (80.0, magenta), (100.0, red)]
                        .iter()
                        .collect(),
                }),
                Box::new(CpuConfig {
                    min_max_width: 40.0,
                    avg_width: 80.0,
                    spacing: 10.0,
                    height: 16.0,
                    colormap: [
                        (0.0, bg2),
                        (0.2, bg1),
                        (0.4, aqua),
                        (0.8, magenta),
                        (1.0, red),
                    ]
                    .iter()
                    .collect(),
                }),
                Box::new(MemoryConfig {
                    colormap: [(1e9, red), (3e9, magenta), (6e9, blue), (8e9, aqua)]
                        .iter()
                        .collect(),
                }),
            ],
            right: vec![
                Box::new(WindowDiagramConfig {
                    border: Color::from_str("#aaaaaa").unwrap(),
                    focused: aqua,
                    active: blue,
                    urgent: red,
                    visible: Color::from_str("#666666").unwrap(),
                }),
                Box::new(NetworkConfig {
                    colormap: [
                        (0.0, bg2),
                        (1e3, bg1),
                        (10e3, aqua),
                        (100e3, blue),
                        (1e6, magenta),
                        (50e6, red),
                    ]
                    .iter()
                    .collect(),
                    spacing: 20.0,
                }),
                Box::new(BatteryConfig {
                    width: 40.0,
                    height: 16.0,
                    spacing: 12.0,
                    colors: BatteryColors {
                        charge: aqua,
                        discharge: red,
                        unknown: magenta,
                    },
                    colormap: [(0.0, red), (0.3, magenta), (0.7, blue), (1.0, aqua)]
                        .iter()
                        .collect(),
                }),
                Box::new(ClockConfig {
                    format: "%a %Y-%m-%d".into(),
                    color: Color::from_str("#4f97d7").unwrap(),
                }),
                Box::new(ClockConfig {
                    format: "%H:%M:%S".into(),
                    color: Color::from_str("#2d9574").unwrap(),
                }),
            ],
        }
    }
}
