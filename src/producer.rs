mod battery;
mod clock;
mod stdin;
mod system;

pub use system::SystemInfo;

use chrono::{DateTime, Local};
use std::sync::{Arc, LazyLock};
use tokio::sync::{Notify, RwLock};

pub static CLOCK: LazyLock<SingleQueue<Arc<DateTime<Local>>>> =
    LazyLock::new(|| clock::Clock::spawn());
pub static STDIN: LazyLock<SingleQueue<Arc<String>>> = LazyLock::new(|| stdin::Stdin::spawn());
pub static SYSTEM: LazyLock<SingleQueue<Arc<SystemInfo>>> =
    LazyLock::new(|| system::System::spawn());
pub static BATTERY: LazyLock<SingleQueue<Arc<(f32, battery::State)>>> =
    LazyLock::new(|| battery::Battery::spawn());

#[allow(async_fn_in_trait)]
pub trait Producer: Default + Send + Sync + 'static {
    type Output: Send + Sync + 'static;
    async fn produce(&mut self) -> Arc<Self::Output>;
    fn initial_value(&mut self) -> Arc<Self::Output>;

    fn spawn() -> SingleQueue<Arc<Self::Output>> {
        let mut producer = Self::default();
        let initial_value = producer.initial_value();
        let queue = SingleQueue::new(initial_value);
        let queue_clone = queue.clone();

        tokio::task::spawn_local(async move {
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
