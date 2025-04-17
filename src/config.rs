use iced::Color;
use serde::{Deserialize, Deserializer};

use crate::{
    consumer::{
        battery::{self, BatteryColors, BatteryConfig},
        clock::ClockConfig,
        cpu::CpuConfig,
        memory::MemoryConfig,
        network::NetworkConfig,
        temp::TempConfig,
    },
    ConsumerConfig,
};

#[derive(Deserialize)]
#[serde(default)]
pub struct RustybarConfig {
    pub global: GlobalConfig,
    pub left: Vec<ConsumerConfig>,
    pub center: Vec<ConsumerConfig>,
    pub right: Vec<ConsumerConfig>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct GlobalConfig {
    pub height: u32,
    #[serde(deserialize_with = "de_color")]
    pub background: Color,
    pub font_size: f32,
    pub spacing: f32,
}

pub fn de_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    let Some(value) = Color::parse(s) else {
        return Err(serde::de::Error::custom("Could not parse as hex color"));
    };

    Ok(value)
}

impl Default for RustybarConfig {
    fn default() -> Self {
        // Spacemacs dark colors
        let bg1 = Color::parse("#292b2e").unwrap();
        let bg2 = Color::parse("#1f2022").unwrap();
        let bg4 = Color::parse("#0a0814").unwrap();
        let aqua = Color::parse("#2d9574").unwrap();
        let blue = Color::parse("#4f97d7").unwrap();
        let magenta = Color::parse("#a31db1").unwrap();
        let red = Color::parse("#f2241f").unwrap();

        Self {
            global: GlobalConfig {
                height: 24,
                background: Color::parse("#000000").unwrap(),
                font_size: 16.0,
                spacing: 12.0,
            },
            left: vec![
                ClockConfig {
                    format: "%a %Y-%m-%d".into(),
                    color: Color::parse("#4f97d7").unwrap(),
                }
                .into(),
                ClockConfig {
                    format: "%H:%M:%S".into(),
                    color: Color::parse("#2d9574").unwrap(),
                }
                .into(),
            ],
            center: vec![
                TempConfig {
                    colormap: [(40.0, aqua), (60.0, blue), (80.0, magenta), (100.0, red)]
                        .iter()
                        .collect(),
                }
                .into(),
                CpuConfig {
                    min_max_width: 40.0,
                    avg_width: 80.0,
                    spacing: 10.0,
                    height: 18.0,
                    colormap: [
                        (0.0, bg2),
                        (0.2, bg1),
                        (0.4, aqua),
                        (0.8, magenta),
                        (1.0, red),
                    ]
                    .iter()
                    .collect(),
                }
                .into(),
                MemoryConfig {
                    colormap: [(1e9, red), (3e9, blue), (8e9, aqua)].iter().collect(),
                }
                .into(),
            ],
            right: vec![
                NetworkConfig {
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
                    spacing: 12.0,
                }
                .into(),
                ClockConfig {
                    format: "%a %Y-%m-%d".into(),
                    color: Color::parse("#4f97d7").unwrap(),
                }
                .into(),
                BatteryConfig {
                    width: 40.0,
                    height: 18.0,
                    spacing: 10.0,
                    colors: BatteryColors {
                        charge: aqua,
                        discharge: red,
                        unknown: magenta,
                    },
                    colormap: [(0.0, red), (0.3, magenta), (0.7, blue), (1.0, aqua)]
                        .iter()
                        .collect(),
                }
                .into(),
                ClockConfig {
                    format: "%H:%M:%S".into(),
                    color: Color::parse("#2d9574").unwrap(),
                }
                .into(),
            ],
        }
    }
}
