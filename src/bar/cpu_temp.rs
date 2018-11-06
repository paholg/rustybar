use crate::bar::{Bar, WriteBar, Writer};
use crate::colormap::{ColorMap, ColorMapConfig};
use failure;
use lazy_static;
use regex;
use std::string::String;
use std::{io::Write, process, time::Duration};

#[derive(Debug, Deserialize)]
pub struct CpuTempConfig {
    colormap: ColorMapConfig,
    min: f32,
    max: f32,
    width: u32,
    height: u32,
}

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
#[derive(Debug)]
pub struct CpuTemp {
    cmap: ColorMap,
    pub min: f32,
    pub max: f32,
    pub width: u32,
    pub height: u32,
}

impl CpuTemp {
    pub fn from_config(
        config: &CpuTempConfig,
        _char_width: u32,
    ) -> Result<CpuTemp, failure::Error> {
        Ok(CpuTemp {
            min: config.min,
            max: config.max,
            width: config.width,
            height: config.height,
            cmap: ColorMap::from_config(&config.colormap)?,
        })
    }
}

impl Bar for CpuTemp {
    fn len(&self) -> u32 {
        self.width
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        lazy_static! {
            static ref RE: regex::Regex = regex::Regex::new(r"(\d+\.\d+)\s*degrees.*").unwrap();
        }
        let info = process::Command::new("acpi").arg("-t").output()?;
        if !info.status.success() {
            bail!(
                "\"acpi -t\" returned exit signal {}. Error: {}",
                info.status,
                String::from_utf8(info.stderr)?,
            );
        }
        let output = String::from_utf8(info.stdout)?;
        let temp: f32 = RE
            .captures(&output)
            .and_then(|c| c.get(1))
            .unwrap()
            .as_str()
            .parse()?;
        let val: f32 = if temp > self.max {
            1.0
        } else if temp < self.min {
            0.0
        } else {
            (temp - self.min) / (self.max - self.min)
        };
        w.bar(
            val,
            self.cmap.map((val * 100.) as u8),
            self.width,
            self.height,
        )?;
        w.write(b"\n")?;
        Ok(())
    }
}
