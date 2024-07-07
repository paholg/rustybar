pub use battery::State;

use std::{sync::Arc, time::Duration};

// TODO: support multiple batteries
pub struct Battery;

impl Battery {
    fn get(&self) -> (f32, State) {
        self.get_inner().unwrap_or((0.0, State::Unknown))
    }

    fn get_inner(&self) -> Option<(f32, State)> {
        // TODO: don't construct manager every read.
        let manager = battery::Manager::new().unwrap();
        let battery = manager.batteries().ok()?.next()?.ok()?;
        Some((battery.state_of_charge().value, battery.state()))
    }
}

impl Default for Battery {
    fn default() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl super::Producer for Battery {
    type Output = (f32, battery::State);

    fn initial_value(&mut self) -> Arc<Self::Output> {
        Arc::new(self.get())
    }

    async fn produce(&mut self) -> Arc<Self::Output> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Arc::new(self.get())
    }
}
