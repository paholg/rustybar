use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use sysinfo::{self, ComponentExt, NetworkExt, NetworksExt, ProcessorExt, SystemExt};

pub struct System {
    sysinfo: sysinfo::System,
    last_tick: Instant,
}

impl System {
    fn get(&mut self) -> SystemInfo {
        let (bytes_received, bytes_transmitted) = self
            .sysinfo
            .get_networks()
            .iter()
            .map(|(_name, network)| (network.get_received(), network.get_transmitted()))
            .fold((0, 0), |sum, (r, t)| (sum.0 + r, sum.1 + t));

        let temperature = self
            .sysinfo
            .get_components()
            .iter()
            .map(|c| c.get_temperature())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(-1.0);
        let available_memory = self.sysinfo.get_available_memory() * 1000;

        let cpus = self
            .sysinfo
            .get_processors()
            .iter()
            .map(|p| p.get_cpu_usage() / 100.0)
            .collect();

        let global_cpu = self.sysinfo.get_global_processor_info().get_cpu_usage() / 100.0;

        let now = Instant::now();
        let tick_duration = now.duration_since(self.last_tick);
        self.last_tick = now;

        SystemInfo {
            tick_duration,
            bytes_received,
            bytes_transmitted,
            temperature,
            available_memory,
            cpus,
            global_cpu,
        }
    }
}

impl Default for System {
    fn default() -> Self {
        let sysinfo = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new()
                .with_components()
                .with_components_list()
                .with_cpu()
                .with_memory()
                .with_networks()
                .with_networks_list(),
        );
        let last_tick = Instant::now();
        Self { sysinfo, last_tick }
    }
}

#[derive(Default)]
pub struct SystemInfo {
    /// Duration since the last update.
    pub tick_duration: Duration,
    /// Bytes recieved across all network interfaces since last update.
    pub bytes_received: u64,
    /// Bytes transmitted across all network interfaces since last update.
    pub bytes_transmitted: u64,
    /// Maximum temperature of all cpus.
    pub temperature: f32,
    /// Fractional usage for all cpus in the range [0.0, 1.0].
    pub cpus: Vec<f32>,
    /// Fractional usage for overall cpu in the range [0.0, 1.0].
    pub global_cpu: f32,
    /// Free memory in bytes
    pub available_memory: u64,
}

#[async_trait::async_trait]
impl super::Producer for System {
    type Output = SystemInfo;

    fn initial_value(&mut self) -> Arc<Self::Output> {
        Arc::new(self.get())
    }

    async fn produce(&mut self) -> Arc<Self::Output> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.sysinfo.refresh_all();
        Arc::new(self.get())
    }
}
