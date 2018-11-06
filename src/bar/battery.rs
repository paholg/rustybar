use crate::colormap::{ColorMap, ColorMapConfig};

use failure;
use std::{fmt, fs, io::Read, io::Write, path::PathBuf, process};

use crate::bar::{Bar, WriteBar, Writer};

#[derive(Debug, Deserialize)]
pub struct BatteryConfig {
    #[serde(default)]
    battery_number: u32,
    #[serde(default)]
    colormap: ColorMapConfig,
    height: u32,
    space: u32,
    width: u32,
}

/// A statusbar for battery information. All data is gathered from '/sys/class/power_supply/BAT#/'
#[derive(Debug, Clone)]
pub struct Battery {
    bat_num: u32,
    path_capacity: PathBuf,
    path_status: PathBuf,
    cmap: ColorMap,
    // the size of bars and spaces between them
    char_width: u32,
    width: u32,
    space: u32,
    height: u32,
}

impl Battery {
    pub fn temp(char_width: u32) -> Result<Battery, failure::Error> {
        let (path_capacity, path_status) = Battery::paths(0)?;
        Ok(Battery {
            bat_num: 0,
            path_capacity: path_capacity,
            path_status: path_status,
            cmap: ColorMap::from_config(&Default::default())?,
            char_width: char_width,
            width: 100,
            space: 20,
            height: 20,
        })
    }

    pub fn from_config(config: &BatteryConfig, char_width: u32) -> Result<Battery, failure::Error> {
        let (path_capacity, path_status) = Battery::paths(config.battery_number)?;
        Ok(Battery {
            bat_num: config.battery_number,
            path_capacity: path_capacity,
            path_status: path_status,
            cmap: ColorMap::from_config(&config.colormap)?,
            char_width: char_width,
            width: config.width,
            space: config.space,
            height: config.height,
        })
    }

    fn paths(bat_num: u32) -> Result<(PathBuf, PathBuf), failure::Error> {
        let base_path: PathBuf = format!("/sys/class/power_supply/BAT{}/", bat_num).into();
        if !base_path.exists() {
            bail!(
                "The selected battery directory:\n\t{}\ndoes not exist and is needed for the battery bar. Perhaps you meant a different battery number or perhaps it isn't there. Go ahead and report this with the output of \"ls /sys/class/power_supply/",
                base_path.display()
            );
        }
        let mut path_capacity = base_path.clone();
        path_capacity.push("capacity");
        if !path_capacity.exists() {
            bail!(
                "The file:\n\t{}\ndoes not exist. It almost certainly should and is needed for the battery bar.",
                path_capacity.display()
            );
        }

        let mut path_status = base_path.clone();
        path_status.push("status");
        if !path_status.exists() {
            bail!(
                "The file:\n\t{}\ndoes not exist. It almost certainly should and is needed for the battery bar.",
                path_status.display()
            );
        }

        Ok((path_capacity, path_status))
    }
}

impl Bar for Battery {
    fn len(&self) -> u32 {
        self.width + self.space + self.char_width
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        let cap_string = {
            let mut string = String::new();
            fs::File::open(&self.path_capacity)?.read_to_string(&mut string)?;
            string
        };
        let capacity: u8 = cap_string.trim().parse()?;

        w.bar(
            (capacity as f32) / 100.,
            self.cmap.map(capacity),
            self.width,
            self.height,
        )?;
        w.space(self.space)?;

        let status_string = {
            let mut string = String::new();
            fs::File::open(&self.path_status)?.read_to_string(&mut string)?;
            string
        };

        let status = match status_string.trim() {
            "Charging" => "^fg(#00ff00)+",
            "Discharging" => "^fg(#ff0000)-",
            "Full" => " ",
            _ => "^fg(#00ffff)*",
        };
        w.write(status.as_bytes())?;
        w.write(b"\n")?;

        Ok(())
    }
}
