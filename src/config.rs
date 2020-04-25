use crate::Font;
use tokio::sync;

pub struct Config {
    pub font: Font,
    pub height: u32,
    pub background: &'static str,
}

impl Config {
    fn new() -> Config {
        Config {
            height: 22,
            font: Font::new("Monospace-12", 12),
            background: "#222222",
        }
    }
}

lazy_static::lazy_static! {
    static ref CONFIG: sync::RwLock<Config> = sync::RwLock::new(Config::new());
}

pub async fn read<'a>() -> sync::RwLockReadGuard<'a, Config> {
    CONFIG.read().await
}

pub async fn write<'a>() -> sync::RwLockWriteGuard<'a, Config> {
    CONFIG.write().await
}
