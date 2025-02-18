use std::sync::Arc;

use crate::{producer::SingleQueue, Color};
use tokio::{io::AsyncWriteExt, process, sync::oneshot, task::JoinHandle};

mod battery;
mod clock;
mod cpu;
mod memory;
mod network;
mod stdin;
mod temp;

pub use self::battery::{Battery, BatteryColors};
pub use clock::Clock;
pub use cpu::Cpu;
pub use memory::Memory;
pub use network::Network;
pub use stdin::Stdin;
pub use temp::Temp;

pub trait Bar: Send + Sized + 'static {
    type Data: Send + Sync;

    /// The width of the bar in pixels.
    fn width(&self) -> u32;

    /// Render the bar as a string.
    fn render(&self, data: &Self::Data) -> String;

    fn data_queue(&self) -> SingleQueue<Arc<Self::Data>>;

    fn start(self) -> BarData {
        let data_queue = self.data_queue();
        let params = BarParams::default();
        let child = start_dzen(&params);
        let width = self.width();

        let (tx, rx) = oneshot::channel();

        let bar = RunningBar {
            bar: self,
            child,
            data_queue,
            init_rx: Some(rx),
        };
        let jh = bar.spawn();
        BarData {
            width,
            init_tx: Some(tx),
            jh,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BarParams {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub bg: Color,
    pub font: String,
}

impl Default for BarParams {
    fn default() -> Self {
        Self {
            x: 0,
            y: 100,
            w: 0,
            h: 0,
            bg: "#000000".into(),
            font: "Monospace-12".into(),
        }
    }
}

pub struct RunningBar<B: Bar> {
    bar: B,
    child: process::Child,
    data_queue: SingleQueue<Arc<B::Data>>,
    init_rx: Option<oneshot::Receiver<BarParams>>,
}

pub struct BarData {
    init_tx: Option<oneshot::Sender<BarParams>>,
    jh: JoinHandle<()>,
    width: u32,
}

impl BarData {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub async fn init(&mut self, params: BarParams) {
        if let Some(tx) = self.init_tx.take() {
            tx.send(params).unwrap();
        }
    }
}

impl Drop for BarData {
    fn drop(&mut self) {
        self.jh.abort();
    }
}

impl<B> RunningBar<B>
where
    B: Bar + Send + 'static,
    B::Data: Send + Sync + 'static,
{
    async fn update(&mut self, data: Arc<B::Data>) {
        let mut rendered = self.bar.render(&data);
        rendered.push('\n');
        self.child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(rendered.as_bytes())
            .await
            .unwrap();
    }

    fn spawn(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            let params = self.init_rx.take().unwrap().await.unwrap();
            self.child = start_dzen(&params);
            let data = self.data_queue.read_now().await;
            self.update(data).await;

            loop {
                let data = self.data_queue.read().await;
                self.update(data).await;
            }
        })
    }
}

fn start_dzen(params: &BarParams) -> process::Child {
    process::Command::new("dzen2")
        .args([
            "-dock",
            "-fn",
            &params.font,
            "-x",
            &params.x.to_string(),
            "-y",
            &params.y.to_string(),
            "-w",
            &params.w.to_string(),
            "-h",
            &params.h.to_string(),
            "-bg",
            &params.bg.to_string(),
            "-ta",
            "l",
            "-e",
            "onstart=lower",
            "-xs",
            "0",
        ])
        .kill_on_drop(true)
        .stdin(std::process::Stdio::piped())
        .spawn()
        // fixme unwrap
        .unwrap()
}
