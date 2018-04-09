use colormap::{ColorMap, ColorMapConfig};
use std::string::String;
use std::{process, io::Write, time::Duration};

use failure;
use regex;

use bar::{write_one_bar, write_space, StatusBar};

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
    lspace: u32,
    rspace: u32,
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
            lspace: 0,
            rspace: 0,
        })
    }
}

impl StatusBar for CpuTemp {
    fn run(&self, w: &mut process::ChildStdin) -> Result<(), failure::Error> {
        let re = regex::Regex::new(r"(\d+\.\d+)\s*degrees.*")?;
        loop {
            let info = process::Command::new("acpi").arg("-t").output()?;
            if !info.status.success() {
                bail!(
                    "\"acpi -t\" returned exit signal {}. Error: {}",
                    info.status,
                    String::from_utf8(info.stderr)?,
                );
            }
            let output = String::from_utf8(info.stdout)?;
            let temp: f32 = re.captures(&output)
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
            write_space(w, self.lspace)?;
            write_one_bar(
                w,
                val,
                self.cmap.map((val * 100.) as u8),
                self.width,
                self.height,
            )?;
            write_space(w, self.rspace)?;
            w.write(b"\n")?;
            ::std::thread::sleep(Duration::from_secs(1));
        }
    }

    fn len(&self) -> u32 {
        self.lspace + self.width + self.rspace
    }

    fn get_lspace(&self) -> u32 {
        self.lspace
    }

    fn set_lspace(&mut self, lspace: u32) {
        self.lspace = lspace
    }

    fn set_rspace(&mut self, rspace: u32) {
        self.rspace = rspace
    }
}
