use std::{sync::Arc, time::Duration};

use jiff::Zoned;

#[derive(Default)]
pub struct Clock;

impl super::Producer for Clock {
    type Output = Zoned;

    fn initial_value(&mut self) -> Arc<Self::Output> {
        Arc::new(Zoned::now())
    }

    async fn produce(&mut self) -> Arc<Self::Output> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Arc::new(Zoned::now())
    }
}
