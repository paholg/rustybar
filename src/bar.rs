use async_trait::async_trait;
use tokio::{
    io::{self, AsyncWriteExt},
    process,
};

mod clock;
mod cpu;
mod memory;
pub(crate) mod stdin;
mod temp;

pub use clock::Clock;
pub use cpu::Cpu;
pub use memory::Memory;
pub use stdin::{run, Stdin};
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

    fn update_on(&self) -> UpdateOn {
        UpdateOn::Tick
    }
}

pub(crate) async fn render_bars(bars: impl Iterator<Item = &RunningBar>) -> Vec<String> {
    let mut res = Vec::new();
    for rb in bars {
        let mut string = rb.bar.render().await;
        string.push('\n');
        res.push(string);
    }

    res
}

pub(crate) async fn update_bars(
    bars: impl Iterator<Item = &mut RunningBar>,
    strings: impl Iterator<Item = &String>,
) -> io::Result<()> {
    for (rb, string) in bars.zip(strings) {
        rb.write(string.as_bytes()).await?;
    }

    Ok(())
}

pub type DynBar = Box<dyn Bar + Send + Sync>;

#[derive(Debug)]
pub(crate) struct RunningBar {
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
}

pub fn bar(val: f32, color: crate::color::Color, width: u32, height: u32) -> String {
    let wfill = (val * (width as f32) + 0.5) as u32;
    let wempty = width - wfill;
    format!(
        "^fg({})^r({2}x{1})^ro({3}x{1})",
        color, height, wfill, wempty
    )
}

pub fn space(width: u32) -> String {
    format!("^r({}x0)", width)
}

// TODO
// pub fn sep(height: u32) -> String {
//     format!("^fg({})^r(2x{})", TEXTCOLOR, height)
// }
