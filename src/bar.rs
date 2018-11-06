mod battery;
mod brightness;
mod clock;
mod cpu;
mod cpu_temp;
mod memory;
mod stdin;

pub use self::{
    battery::{Battery, BatteryConfig},
    brightness::{Brightness, BrightnessConfig},
    clock::{Clock, ClockConfig},
    cpu::{Cpu, CpuConfig},
    cpu_temp::{CpuTemp, CpuTempConfig},
    memory::{Memory, MemoryConfig},
    stdin::{Stdin, StdinConfig},
};

use failure;
use std::{fmt, io, io::Write, marker, process, time::Duration};

use crate::color::Color;

pub type Writer = process::ChildStdin;

// fixme: this should be settable
static TEXTCOLOR: &'static str = "#888888";

pub trait Bar: fmt::Debug + marker::Send + marker::Sync {
    /// Give the length in pixels of the output string. This is used to size the dzen2 bar and to
    /// allocate space. It is assumed to be constant.
    fn len(&self) -> u32;

    /// Block the thread until it is time to produce the next output. Default implemention sleeps
    /// for 1 second.
    fn block(&mut self) -> Result<(), failure::Error> {
        ::std::thread::sleep(Duration::from_secs(1));
        Ok(())
    }

    /// Any extra initialization steps can go here.
    fn initialize(&mut self) -> Result<(), failure::Error> {
        Ok(())
    }

    /// Output the bar contents to the writer.
    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error>;
}

#[derive(Debug)]
pub struct BarWithSep {
    bar: Box<dyn Bar>,
    pub left: u32,
    pub right: u32,
}

impl BarWithSep {
    pub fn new(bar: Box<dyn Bar>) -> BarWithSep {
        BarWithSep {
            left: 0,
            bar,
            right: 0,
        }
    }
}

impl Bar for BarWithSep {
    fn len(&self) -> u32 {
        self.left + self.bar.len() + self.right
    }

    fn block(&mut self) -> Result<(), failure::Error> {
        self.bar.block()
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        if self.left > 0 {
            write!(w, "^r({}x0)", self.left)?;
        }

        self.bar.write(w)?;

        if self.right > 0 {
            write!(w, "^r({}x0)", self.right)?;
        }
        w.write_all(b"\n")?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "bar")]
#[allow(non_camel_case_types)]
pub enum BarConfig {
    battery(BatteryConfig),
    brightness(BrightnessConfig),
    clock(ClockConfig),
    cpu(CpuConfig),
    cputemp(CpuTempConfig),
    memory(MemoryConfig),
    stdin(StdinConfig),
}

impl BarConfig {
    pub fn to_bar(&self, char_width: u32) -> Result<Box<dyn Bar>, failure::Error> {
        let bar: Box<dyn Bar> = match self {
            BarConfig::battery(ref b) => Box::new(Battery::from_config(&b, char_width)?),
            BarConfig::brightness(ref b) => Box::new(Brightness::from_config(&b, char_width)?),
            BarConfig::clock(ref b) => Box::new(Clock::from_config(&b, char_width)?),
            BarConfig::cpu(ref b) => Box::new(Cpu::from_config(&b, char_width)?),
            BarConfig::cputemp(ref b) => Box::new(CpuTemp::from_config(&b, char_width)?),
            BarConfig::memory(ref b) => Box::new(Memory::from_config(&b, char_width)?),
            BarConfig::stdin(ref b) => Box::new(Stdin::from_config(&b, char_width)?),
        };

        Ok(bar)
    }
}

pub trait WriteBar {
    fn bar(&mut self, val: f32, color: Color, width: u32, height: u32) -> Result<(), io::Error>;
    fn space(&mut self, width: u32) -> Result<(), io::Error>;
    fn sep(&mut self, height: u32) -> Result<(), io::Error>;
}

impl<W: Write> WriteBar for W {
    fn bar(&mut self, val: f32, color: Color, width: u32, height: u32) -> Result<(), io::Error> {
        let wfill = (val * (width as f32) + 0.5) as u32;
        let wempty = width - wfill;
        write!(
            self,
            "^fg({})^r({2}x{1})^ro({3}x{1})",
            color, height, wfill, wempty
        )
    }

    fn space(&mut self, width: u32) -> Result<(), io::Error> {
        write!(self, "^r({}x0)", width)
    }

    fn sep(&mut self, height: u32) -> Result<(), io::Error> {
        write!(self, "^fg({})^r(2x{})", TEXTCOLOR, height)
    }
}
