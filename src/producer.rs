mod battery;
mod clock;
mod stdin;
mod system;

pub use system::SystemInfo;

use async_trait::async_trait;
use chrono::{DateTime, Local};
use lazy_static::lazy_static;
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};

lazy_static! {
    pub static ref CLOCK: SingleQueue<Arc<DateTime<Local>>> = clock::Clock::spawn();
    pub static ref STDIN: SingleQueue<Arc<String>> = stdin::Stdin::spawn();
    pub static ref SYSTEM: SingleQueue<Arc<SystemInfo>> = system::System::spawn();
    pub static ref BATTERY: SingleQueue<Arc<(f32, battery::State)>> = battery::Battery::spawn();
}

#[async_trait]
pub trait Producer: Default + Send + Sync + 'static {
    type Output: Send + Sync + 'static;
    async fn produce(&mut self) -> Arc<Self::Output>;
    fn initial_value(&mut self) -> Arc<Self::Output>;

    fn spawn() -> SingleQueue<Arc<Self::Output>> {
        let mut producer = Self::default();
        let initial_value = producer.initial_value();
        let queue = SingleQueue::new(initial_value);
        let queue_clone = queue.clone();

        tokio::spawn(async move {
            loop {
                let data = producer.produce().await;
                queue.write(data).await;
            }
        });

        queue_clone
    }
}

#[derive(Clone, Default)]
pub struct SingleQueue<T> {
    lock: Arc<RwLock<T>>,
    notifier: Arc<Notify>,
}

impl<T: Clone> SingleQueue<T> {
    pub fn new(data: T) -> Self {
        Self {
            lock: Arc::new(RwLock::new(data)),
            notifier: Default::default(),
        }
    }

    pub async fn write(&self, data: T) {
        let mut guard = self.lock.write().await;
        *guard = data;
        drop(guard);
        self.notifier.notify_waiters();
    }

    pub async fn read(&self) -> T {
        self.notifier.notified().await;
        self.read_now().await
    }

    pub async fn read_now(&self) -> T {
        self.lock.read().await.clone()
    }
}
