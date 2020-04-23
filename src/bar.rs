use async_trait::async_trait;
use tokio::{
    io::{self, AsyncWriteExt},
    process,
};

mod clock;

pub use clock::Clock;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UpdateOn {
    Tick,
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
    pub fn start(bar: DynBar, font: &str, x: u32, w: u32, h: u32) -> RunningBar {
        let process = process::Command::new("dzen2")
            .args(&[
                "-dock",
                "-fn",
                font,
                "-x",
                &x.to_string(),
                "-w",
                &w.to_string(),
                "-h",
                &h.to_string(),
                "-bg",
                "#000000",
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

pub struct RustyBar {
    screen_id: u32,
    left: Vec<DynBar>,
    center: Vec<DynBar>,
    right: Vec<DynBar>,
}

impl RustyBar {
    pub fn new(
        screen_id: u32,
        left: Vec<DynBar>,
        center: Vec<DynBar>,
        right: Vec<DynBar>,
    ) -> RustyBar {
        RustyBar {
            screen_id,
            left,
            center,
            right,
        }
    }

    pub async fn run(&self) {
        for rb in (self.left.iter())
            .chain(self.center.iter())
            .chain(self.right.iter())
            .map(|bar| RunningBar::start(bar.box_clone(), "Monospace-9", 200, 300, 18))
        {
            match rb.bar.update_on() {
                UpdateOn::Tick => crate::state::write().await.register_bar(rb),
                UpdateOn::Custom => (), // Do nothing here; it's up to the user to handle
            }
        }
    }
}
