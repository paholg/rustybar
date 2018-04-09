use colormap::{ColorMap, ColorMapConfig};

use failure;
use std::{fs, process, io::Read, io::Write, path::PathBuf};
use std::time::Duration;

use bar::{write_one_bar, write_space, StatusBar};

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
    pub width: u32,
    pub space: u32,
    pub height: u32,
    pub lspace: u32,
    pub rspace: u32,
}

impl Battery {
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
            lspace: 0,
            rspace: 0,
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

impl StatusBar for Battery {
    fn run(&self, w: &mut process::ChildStdin) -> Result<(), failure::Error> {
        loop {
            let cap_string = {
                let mut string = String::new();
                fs::File::open(&self.path_capacity)?.read_to_string(&mut string)?;
                string
            };
            let capacity: u8 = cap_string.trim().parse()?;

            write_space(w, self.lspace)?;
            write_one_bar(
                w,
                (capacity as f32) / 100.,
                self.cmap.map(capacity),
                self.width,
                self.height,
            )?;
            write_space(w, self.space)?;

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
            write_space(w, self.rspace)?;
            w.write(b"\n")?;

            ::std::thread::sleep(Duration::from_secs(1));
        }
    }

    fn len(&self) -> u32 {
        let len = self.lspace + self.width + self.space + self.char_width + self.rspace;
        println!("bat {} len {}", self.bat_num, len);
        len
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
