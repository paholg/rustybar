use crate::bar::RunningBar;
use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::stream::StreamExt;
use tokio::sync::{Mutex, RwLock};

lazy_static! {
    static ref STDIN_STATE: RwLock<StdinState> = RwLock::new(StdinState::new());
    static ref LINES: Mutex<io::Lines<io::BufReader<io::Stdin>>> =
        Mutex::new(io::BufReader::new(io::stdin()).lines());
}

#[derive(Debug)]
pub struct StdinState {
    buf: String,
    bars_to_update: Vec<RunningBar>,
    running: bool,
}

impl StdinState {
    fn new() -> Self {
        Self {
            buf: String::new(),
            bars_to_update: Vec::new(),
            running: false,
        }
    }
}

#[derive(Debug)]
pub struct Stdin;

impl Stdin {
    pub async fn line(&self) -> String {
        STDIN_STATE.read().await.buf.clone()
    }
}

#[async_trait]
impl crate::updater::Updater for Stdin {
    async fn register(&self, bar: RunningBar) {
        STDIN_STATE.write().await.bars_to_update.push(bar);
    }

    async fn clear(&self) {
        STDIN_STATE.write().await.bars_to_update.clear();
    }

    async fn update_state(&self) {
        let line = LINES.lock().await.next().await.unwrap().unwrap();

        STDIN_STATE.write().await.buf = line;
    }

    async fn mark_running(&self) {
        STDIN_STATE.write().await.running = true;
    }

    async fn running(&self) -> bool {
        STDIN_STATE.read().await.running
    }

    async fn run(&self) {
        if self.running().await {
            return;
        }
        self.mark_running().await;

        let stdin = Stdin;

        tokio::spawn(async move {
            loop {
                stdin.update_state().await;
                let strings =
                    crate::updater::render_bars(STDIN_STATE.read().await.bars_to_update.iter())
                        .await;
                crate::updater::update_bars(
                    STDIN_STATE.write().await.bars_to_update.iter_mut(),
                    strings.iter(),
                )
                .await
                // TODO: remove unwrap
                .unwrap();
            }
        });
    }
}
