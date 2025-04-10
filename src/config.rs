use std::sync::LazyLock;

use crate::{Color, Font};
use tokio::sync;

#[derive(Clone, Debug)]
pub struct Config {
    pub font: Font,
    pub height: u32,
    pub background: Color,
}

impl Config {
    fn new() -> Config {
        Config {
            height: 22,
            font: Font::new("Monospace-12".into(), 12),
            background: "#222222".into(),
        }
    }
}

static CONFIG: LazyLock<sync::RwLock<Config>> = LazyLock::new(|| sync::RwLock::new(Config::new()));

pub async fn get() -> Config {
    CONFIG.read().await.clone()
}

pub async fn write<'a>() -> sync::RwLockWriteGuard<'a, Config> {
    CONFIG.write().await
}
