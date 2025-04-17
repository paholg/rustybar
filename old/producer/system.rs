use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use sysinfo::{self, CpuRefreshKind, MemoryRefreshKind};

pub struct System {
    system: sysinfo::System,
    networks: sysinfo::Networks,
    components: sysinfo::Components,
    last_tick: Instant,
}

impl System {
    fn get(&mut self) -> SystemInfo {
        let (bytes_received, bytes_transmitted) = self
            .networks
            .iter()
            .map(|(_name, network)| (network.received(), network.transmitted()))
            .fold((0, 0), |sum, (r, t)| (sum.0 + r, sum.1 + t));

        let temperature = self
            .components
            .iter()
            .flat_map(|c| c.temperature())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(-1.0);
        let available_memory = self.system.available_memory();

        let cpus = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() / 100.0)
            .collect();

        let global_cpu = self.system.global_cpu_usage() / 100.0;

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
        Self {
            last_tick: Instant::now(),
            system: sysinfo::System::new(),
            networks: sysinfo::Networks::new(),
            components: sysinfo::Components::new(),
        }
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

impl super::Producer for System {
    type Output = SystemInfo;

    fn initial_value(&mut self) -> Arc<Self::Output> {
        Arc::new(self.get())
    }

    async fn produce(&mut self) -> Arc<Self::Output> {
        tokio::time::sleep(Duration::from_secs(1)).await;

        self.system.refresh_specifics(
            sysinfo::RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        self.networks.refresh(true);
        self.components.refresh(true);

        Arc::new(self.get())
    }
}
