use std::time::{Duration, Instant};

use jiff::Zoned;
use starship_battery::State;
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};
use tokio::time::sleep;

use crate::Message;

use super::Producer;

#[derive(Debug, Default, Clone)]
pub struct Battery {
    pub charge: f32,
    pub state: State,
}

#[derive(Debug, Default, Clone)]
pub struct Network {
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
}

#[derive(Debug, Default, Clone)]
pub struct Cpu {
    pub min: f32,
    pub avg: f32,
    pub max: f32,
}

#[derive(Debug, Default, Clone)]
pub struct Temperature {
    pub max: f32,
}

#[derive(Debug, Default, Clone)]
pub struct Memory {
    pub available: u64,
}

#[derive(Debug)]
pub struct TickMessage {
    /// Duration since the last update.
    pub tick_duration: Duration,
    /// The time at the last update.
    pub time: Zoned,
    pub battery: Battery,
    pub network: Network,
    pub cpu: Cpu,
    pub temp: Temperature,
    pub memory: Memory,
}

/// This producer currently produces anything that we produce on a 1-second
/// tick.
pub struct TickProducer {
    last_tick: Instant,
    system: System,
    networks: Networks,
    components: Components,
    battery_manager: starship_battery::Manager,
}

impl Default for TickProducer {
    fn default() -> Self {
        Self {
            last_tick: Instant::now(),
            system: System::default(),
            networks: Networks::default(),
            components: Components::default(),
            battery_manager: starship_battery::Manager::new().unwrap(),
        }
    }
}

impl TickProducer {
    fn battery(&self) -> Option<Battery> {
        let battery = self.battery_manager.batteries().ok()?.next()?.ok()?;
        Some(Battery {
            charge: battery.state_of_charge().value,
            state: battery.state(),
        })
    }

    fn network(&self) -> Network {
        let (bytes_received, bytes_transmitted) = self
            .networks
            .iter()
            .filter_map(|(name, network)| {
                if name.starts_with("lo") {
                    None
                } else {
                    Some((network.received(), network.transmitted()))
                }
            })
            .fold((0, 0), |sum, (r, t)| (sum.0 + r, sum.1 + t));

        Network {
            bytes_received,
            bytes_transmitted,
        }
    }

    fn cpu(&self) -> Cpu {
        let (min, max) = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() / 100.0)
            .fold((f32::MAX, f32::MIN), |(min, max), cpu| {
                (min.min(cpu), max.max(cpu))
            });
        let avg = self.system.global_cpu_usage() / 100.0;

        Cpu { min, avg, max }
    }

    fn temp(&self) -> Temperature {
        let max = self
            .components
            .iter()
            .flat_map(|c| c.temperature())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(-1.0);
        Temperature { max }
    }

    fn memory(&self) -> Memory {
        let available = self.system.available_memory();
        Memory { available }
    }
}

impl Producer for TickProducer {
    async fn produce(&mut self) -> Message {
        sleep(std::time::Duration::from_secs(1)).await;

        self.system.refresh_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::nothing().with_ram()),
        );
        self.networks.refresh(true);
        self.components.refresh(true);

        let now = Instant::now();
        let tick_duration = now.duration_since(self.last_tick);
        self.last_tick = now;

        Message::Tick(TickMessage {
            time: Zoned::now(),
            tick_duration,
            battery: self.battery().unwrap_or_default(),
            network: self.network(),
            cpu: self.cpu(),
            temp: self.temp(),
            memory: self.memory(),
        })
    }
}
