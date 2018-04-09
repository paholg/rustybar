mod battery;
mod clock;
mod cpu;
mod cpu_temp;
// mod memory;
mod stdin;

pub use self::{battery::{Battery, BatteryConfig}, clock::{Clock, ClockConfig},
               cpu::{Cpu, CpuConfig}, cpu_temp::{CpuTemp, CpuTempConfig},
               stdin::{Stdin, StdinConfig}};

use std::{fmt, marker, process, io::Write};
use failure;

use colormap::Color;

// fixme: this should be settable
static TEXTCOLOR: &'static str = "#888888";

pub trait StatusBar: fmt::Debug + marker::Send + marker::Sync {
    fn run(&self, w: &mut process::ChildStdin) -> Result<(), failure::Error>;
    /// Give the length in pixels of the output string.
    fn len(&self) -> u32;
    fn get_lspace(&self) -> u32;
    fn set_lspace(&mut self, lspace: u32);
    fn set_rspace(&mut self, rspace: u32);
}

#[derive(Debug, Deserialize)]
#[serde(tag = "bar")]
#[allow(non_camel_case_types)]
pub enum BarConfig {
    battery(BatteryConfig),
    clock(ClockConfig),
    cpu(CpuConfig),
    cputemp(CpuTempConfig),
    // memory(MemoryConfig),
    stdin(StdinConfig),
}

impl BarConfig {
    pub fn into_bar(&self, char_width: u32) -> Result<Box<StatusBar>, failure::Error> {
        let bar: Box<StatusBar> = match self {
            &BarConfig::battery(ref b) => Box::new(Battery::from_config(&b, char_width)?),
            &BarConfig::clock(ref b) => Box::new(Clock::from_config(&b, char_width)?),
            &BarConfig::cpu(ref b) => Box::new(Cpu::from_config(&b, char_width)?),
            &BarConfig::cputemp(ref b) => Box::new(CpuTemp::from_config(&b, char_width)?),
            // &BarConfig::memory(ref b) => Box::new(Memory::from_config(&b, char_width)?),
            &BarConfig::stdin(ref b) => Box::new(Stdin::from_config(&b, char_width)?),
        };

        Ok(bar)
    }
}

pub fn write_one_bar(
    w: &mut process::ChildStdin,
    val: f32,
    color: Color,
    width: u32,
    height: u32,
) -> Result<(), failure::Error> {
    // fixme: .round()?
    let wfill = (val * (width as f32) + 0.5) as u32;
    let wempty = width - wfill;
    write!(
        w,
        "^fg({})^r({2}x{1})^ro({3}x{1})",
        color, height, wfill, wempty
    )?;
    Ok(())
}

pub fn write_space(w: &mut process::ChildStdin, width: u32) -> Result<(), failure::Error> {
    write!(w, "^r({}x0)", width)?;
    Ok(())
}

pub fn write_sep(w: &mut process::ChildStdin, height: u32) -> Result<(), failure::Error> {
    write!(w, "^fg({})^r(2x{})", TEXTCOLOR, height)?;
    Ok(())
}
