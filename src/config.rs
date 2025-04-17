use iced::{Color, Pixels};
use serde::{Deserialize, Deserializer};

use crate::{
    consumer::{clock::ClockConfig, cpu::CpuConfig},
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
                font_size: 18.0,
                spacing: 12.0,
            },
            left: vec![],
            center: vec![CpuConfig {
                min_max_width: 40.0,
                avg_width: 80.0,
                spacing: 8.0,
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
            }
            .into()],
            right: vec![
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
        }
    }
}
