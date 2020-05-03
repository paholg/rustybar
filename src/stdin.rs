use crate::bar::RunningBar;
use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::stream::StreamExt;
use tokio::sync::RwLock;

lazy_static! {
    static ref STDIN_STATE: RwLock<StdinState> = RwLock::new(StdinState::new());
}

#[derive(Debug)]
pub struct StdinState {
    buf: String,
    bars_to_update: Vec<RunningBar>,
    lines: io::Lines<io::BufReader<io::Stdin>>,
}

impl StdinState {
    fn new() -> Self {
        Self {
            buf: String::new(),
            bars_to_update: Vec::new(),
            lines: io::BufReader::new(io::stdin()).lines(),
        }
    }
}

pub struct Stdin;

impl Stdin {}

// #[async_trait]
// impl crate::updater::Updater for Stdin {
//     async fn register(&self, bar: RunningBar) {
//         STDIN_STATE.write().await.bars_to_update.push(bar);
//     }

//     async fn clear(&self) {
//         STDIN_STATE.write().await.bars_to_update.clear();
//     }

//     async fn update_state(&self) {
//         let mut lock = STDIN_STATE.write().await;
//         lock.buf = lock.lines.next().await.unwrap().unwrap();
//     }

//     async fn mark_running(&self) {
//         STDIN_STATE.
//     }
// }
