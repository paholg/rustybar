use async_trait::async_trait;
use tokio::{
    io::{self, AsyncWriteExt},
    process,
};

mod clock;
mod cpu;
mod memory;
pub mod stdin;
mod temp;

use crate::ticker::Ticker;
use crate::updater::Updater;
pub use clock::Clock;
pub use cpu::Cpu;
pub use memory::Memory;
pub use stdin::Stdin;
pub use temp::Temp;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UpdateOn {
    Tick,
    Stdin,
    Custom,
}

#[async_trait]
pub trait Bar: std::fmt::Debug {
    /// The width of the bar in pixels.
    fn width(&self) -> u32;

    /// Render the bar as a string.
    async fn render(&self) -> String;

    fn box_clone(&self) -> DynBar;

    fn updater(&self) -> Box<dyn Updater> {
        Box::new(Ticker)
    }
}

pub type DynBar = Box<dyn Bar + Send + Sync>;

#[derive(Debug)]
pub struct RunningBar {
    pub bar: DynBar,
    child: process::Child,
}

impl RunningBar {
    pub fn start(bar: DynBar, font: &str, x: u32, y: u32, w: u32, h: u32, bg: &str) -> RunningBar {
        let process = process::Command::new("dzen2")
            .args(&[
                "-dock",
                "-fn",
                font,
                "-x",
                &x.to_string(),
                "-y",
                &y.to_string(),
                "-w",
                &w.to_string(),
                "-h",
                &h.to_string(),
                "-bg",
                bg,
                "-ta",
                "l",
                "-e",
                "onstart=raise",
                "-xs",
                "0",
            ])
            .kill_on_drop(true)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        RunningBar {
            bar,
            child: process,
        }
    }

    pub async fn write<'a>(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.child.stdin.as_mut().unwrap().write_all(bytes).await
    }

    pub async fn register(self) {
        let updater = self.bar.updater();
        updater.register(self).await;
        updater.run().await;
    }
}
