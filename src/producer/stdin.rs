use std::sync::Arc;

use tokio::io::{self, AsyncBufReadExt};

pub struct Stdin {
    lines: io::Lines<io::BufReader<io::Stdin>>,
}

impl Default for Stdin {
    fn default() -> Self {
        let lines = io::BufReader::new(io::stdin()).lines();
        Self { lines }
    }
}

impl super::Producer for Stdin {
    type Output = String;

    fn initial_value(&mut self) -> Arc<Self::Output> {
        Default::default()
    }

    async fn produce(&mut self) -> Arc<Self::Output> {
        Arc::new(self.lines.next_line().await.unwrap().unwrap())
    }
}
