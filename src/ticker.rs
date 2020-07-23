use async_trait::async_trait;
use lazy_static::lazy_static;
use std::time::Duration;
use sysinfo::{ComponentExt, NetworkExt, NetworksExt, ProcessorExt, SystemExt};
use tokio::sync::RwLock;

use crate::bar::RunningBar;
use crate::{screen::Screen, RustyBar};

lazy_static! {
    static ref TICKER_STATE: RwLock<TickerState> = RwLock::new(TickerState::new());
}

#[derive(Debug)]
pub struct TickerState {
    /// The time of the last update for things that update every second.
    last_tick: std::time::Instant,
    /// The time between the last two ticks.
    tick_duration: Duration,
    /// The current time, updated every second.
    time: chrono::DateTime<chrono::Local>,
    /// System info, updated every second.
    system: sysinfo::System,
    /// Bytes recieved across all network interfaces since last update.
    bytes_recieved: u64,
    /// Bytes transmitted across all network interfaces since last update.
    bytes_transmitted: u64,
    /// Maximum temperature of all cpus.
    temperature: f32,

    running: bool,

    pub screens: Vec<Screen>,
    bars_to_update: Vec<RunningBar>,

    bar_config: Vec<RustyBar>,
}

impl TickerState {
    fn new() -> Self {
        Self {
            last_tick: std::time::Instant::now(),
            tick_duration: Duration::from_secs(0),
            time: chrono::Local::now(),
            system: sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::new()
                    .with_components()
                    .with_components_list()
                    .with_cpu()
                    .with_memory()
                    .with_networks()
                    .with_networks_list(),
            ),
            bytes_recieved: 0,
            bytes_transmitted: 0,
            temperature: 0.0,
            running: false,
            bars_to_update: Vec::new(),
            screens: crate::screen::get_screens(),

            bar_config: Vec::new(),
        }
    }

    /// Update everything that updates every second.
    async fn on_tick(&mut self) {
        let now = std::time::Instant::now();
        self.tick_duration = now.duration_since(self.last_tick);
        self.last_tick = now;
        self.time = chrono::Local::now();
        self.system.refresh_all();

        let (rec, tran) = self
            .system
            .get_networks()
            .iter()
            .map(|(_name, network)| (network.get_received(), network.get_transmitted()))
            .fold((0, 0), |sum, (r, t)| (sum.0 + r, sum.1 + t));
        self.bytes_recieved = rec;
        self.bytes_transmitted = tran;

        self.temperature = self
            .system
            .get_components()
            .iter()
            .map(|c| c.get_temperature())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(-1.0);

        self.screens = crate::screen::get_screens();
    }
}

#[derive(Debug)]
pub struct Ticker;

impl Ticker {
    /// Bytes recieved across all network interfaces since last update.
    pub async fn bytes_recieved(&self) -> u64 {
        TICKER_STATE.read().await.bytes_recieved
    }

    /// Bytes transmitted across all network interfaces since last update.
    pub async fn bytes_transmitted(&self) -> u64 {
        TICKER_STATE.read().await.bytes_transmitted
    }

    pub async fn tick_duration(&self) -> Duration {
        TICKER_STATE.read().await.tick_duration
    }

    /// Maximum temperature of all cpus.
    pub async fn temperature(&self) -> f32 {
        TICKER_STATE.read().await.temperature
    }

    /// Fractioal usage for all cpus in the range [0.0, 1.0].
    pub async fn cpus(&self) -> Vec<f32> {
        TICKER_STATE
            .read()
            .await
            .system
            .get_processors()
            .iter()
            .map(|p| p.get_cpu_usage() / 100.0)
            .collect()
    }

    /// Fractioal usage for overall cpu in the range [0.0, 1.0].
    pub async fn global_cpu(&self) -> f32 {
        TICKER_STATE
            .read()
            .await
            .system
            .get_global_processor_info()
            .get_cpu_usage()
            / 100.0
    }

    /// Free memory in bytes
    pub async fn free_memory(&self) -> u64 {
        TICKER_STATE.read().await.system.get_free_memory() * 1000
    }

    /// Clock time
    pub async fn time(&self) -> chrono::DateTime<chrono::Local> {
        TICKER_STATE.read().await.time
    }

    pub async fn set_bar_config(&self, bars: Vec<RustyBar>) {
        TICKER_STATE.write().await.bar_config = bars;
    }

    pub async fn restart(&self) {
        let (bars, screens) = {
            let ticker_lock = TICKER_STATE.read().await;
            (ticker_lock.bar_config.clone(), ticker_lock.screens.clone())
        };

        for bar in &bars {
            bar.stop().await;
        }

        for bar in &bars {
            if let Some(screen) = screens.iter().find(|&&screen| screen.id == bar.screen_id) {
                bar.start(screen).await;
            }
        }
    }
}

#[async_trait]
impl crate::updater::Updater for Ticker {
    async fn register(&self, bar: RunningBar) {
        TICKER_STATE.write().await.bars_to_update.push(bar);
    }

    async fn clear(&self) {
        TICKER_STATE.write().await.bars_to_update.clear();
    }

    async fn update_state(&self) {
        let duration = Duration::from_millis(
            1000 - TICKER_STATE.read().await.time.timestamp_subsec_millis() as u64,
        );
        tokio::time::delay_for(duration).await;
        let screens = TICKER_STATE.read().await.screens.clone();
        TICKER_STATE.write().await.on_tick().await;
        let new_screens = TICKER_STATE.read().await.screens.clone();

        if new_screens != screens {
            Ticker.restart().await;
        }
    }

    async fn mark_running(&self) {
        TICKER_STATE.write().await.running = true;
    }

    async fn running(&self) -> bool {
        TICKER_STATE.read().await.running
    }

    async fn run(&self) {
        if self.running().await {
            return;
        }
        self.mark_running().await;

        let ticker = Ticker;

        tokio::spawn(async move {
            loop {
                ticker.update_state().await;
                let strings =
                    crate::updater::render_bars(TICKER_STATE.read().await.bars_to_update.iter())
                        .await;
                crate::updater::update_bars(
                    TICKER_STATE.write().await.bars_to_update.iter_mut(),
                    strings.iter(),
                )
                .await
                // TODO: remove unwrap
                .unwrap();
            }
        });
    }
}
