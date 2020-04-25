use lazy_static::lazy_static;
use std::time::Duration;
use sysinfo::{ComponentExt, NetworkExt, NetworksExt, ProcessorExt, SystemExt};
use tokio::io;
use tokio::sync::{self, RwLock};

use crate::bar::RunningBar;
use crate::screen::Screen;

lazy_static! {
    static ref STATE: RwLock<State> = RwLock::new(State::new());
}

pub async fn read<'a>() -> sync::RwLockReadGuard<'a, State> {
    STATE.read().await
}

pub(crate) async fn write<'a>() -> sync::RwLockWriteGuard<'a, State> {
    STATE.write().await
}

#[derive(Debug)]
pub struct State {
    /// The time of the last update for things that update every second.
    last_tick: std::time::Instant,
    /// The time between the last two ticks.
    tick_duration: Duration,
    stdin: String,
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

    pub screens: Vec<Screen>,
    bars_to_update: Vec<RunningBar>,
}

impl State {
    fn new() -> State {
        State {
            last_tick: std::time::Instant::now(),
            tick_duration: Duration::from_secs(0),
            stdin: String::new(),
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
            temperature: -1.0,
            bars_to_update: Vec::new(),
            screens: crate::screen::get_screens(),
        }
    }

    /// Update everything that updates every second.
    fn on_tick(&mut self) {
        let now = std::time::Instant::now();
        self.tick_duration = now.duration_since(self.last_tick);
        self.last_tick = now;
        self.time = chrono::Local::now();
        dbg!(&self.time);
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
    }

    pub(crate) fn register_bar(&mut self, bar: RunningBar) {
        self.bars_to_update.push(bar);
    }

    pub(crate) fn clear_bars(&mut self) {
        self.bars_to_update.clear();
    }

    /// Bytes recieved across all network interfaces since last update.
    pub fn bytes_recieved(&self) -> u64 {
        self.bytes_recieved
    }

    /// Bytes transmitted across all network interfaces since last update.
    pub fn bytes_transmitted(&self) -> u64 {
        self.bytes_transmitted
    }

    /// Maximum temperature of all cpus.
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Percent usage for all cpus.
    pub fn cpus<'a>(&'a self) -> impl Iterator<Item = f32> + 'a {
        self.system
            .get_processors()
            .iter()
            .map(|p| p.get_cpu_usage() / 100.0)
    }

    /// Free memory in bytes
    pub fn free_memory(&self) -> u64 {
        self.system.get_free_memory() * 1000
    }

    /// Free memory in bytes
    pub fn time(&self) -> chrono::DateTime<chrono::Local> {
        self.time
    }
}

/// Run forever, updating the parts of STATE that should be updated on a regular interval.
pub async fn tick() -> io::Result<()> {
    loop {
        write().await.on_tick();

        let state = read().await;

        let new_screens = crate::screen::get_screens();
        if state.screens != new_screens {
            todo!("recreate bars");
        }

        let strings = crate::bar::render_bars(state.bars_to_update.iter()).await;

        let duration = Duration::from_millis(1000 - state.time().timestamp_subsec_millis() as u64);

        std::mem::drop(state);
        crate::bar::update_bars(write().await.bars_to_update.iter_mut(), strings.iter()).await?;

        tokio::time::delay_for(duration).await;
    }
}
