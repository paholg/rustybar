use chrono::{DateTime, Local};
use std::{sync::Arc, time::Duration};

pub struct Clock;

impl Default for Clock {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl super::Producer for Clock {
    type Output = DateTime<Local>;

    fn initial_value(&mut self) -> Arc<Self::Output> {
        Arc::new(Local::now())
    }

    async fn produce(&mut self) -> Arc<Self::Output> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Arc::new(Local::now())
    }
}
