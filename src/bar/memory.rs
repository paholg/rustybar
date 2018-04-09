use colormap::{ColorMap, ColorMapConfig};

use failure;
use regex;
use std::{fs, path, process, thread, time, io::Read, io::Write};

use bar::{write_one_bar, write_space, StatusBar};

#[derive(Debug, Deserialize)]
pub struct MemoryConfig {
    #[serde(default)]
    colormap: ColorMapConfig,
    width: u32,
    height: u32,
}

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
#[derive(Debug)]
pub struct Memory {
    cmap: ColorMap,
    width: u32,
    height: u32,
    lspace: u32,
    rspace: u32,
}

impl Memory {
    pub fn from_config(config: &MemoryConfig, _char_width: u32) -> Result<Memory, failure::Error> {
        Ok(Memory {
            cmap: ColorMap::from_config(&config.colormap)?,
            width: config.width,
            height: config.height,
            lspace: 0,
            rspace: 0,
        })
    }
}

impl StatusBar for Memory {
    fn run(&self, w: &mut process::ChildStdin) -> Result<(), failure::Error> {
        let path = path::Path::new("/proc/meminfo");
        if !path.is_file() {
            bail!(
                "The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?",
                path.display()
            );
        }
        let re_tot = regex::Regex::new(r"MemTotal.*?(\d+)")?;
        let re_free = regex::Regex::new(r"MemFree.*?(\d+)")?;
        let re_buffers = regex::Regex::new(r"Buffers.*?(\d+)")?;
        let re_cached = regex::Regex::new(r"Cached.*?(\d+)")?;
        let info = {
            let mut buffer = String::new();
            fs::File::open(&path)?.read_to_string(&mut buffer)?;
            buffer
        };

        // let total: f32 = 1.0;
        let total: f32 = re_tot
            .captures(&info)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse()?;

        loop {
            let info = {
                let mut buffer = String::new();
                fs::File::open(&path)?.read_to_string(&mut buffer)?;
                buffer
            };
            let free: f32 = re_free
                .captures(&info)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse()?;
            let buffers: f32 = re_buffers
                .captures(&info)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse()?;
            let cached: f32 = re_cached
                .captures(&info)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse()?;
            let val = (total - free - buffers - cached) / total;

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

            thread::sleep(time::Duration::from_secs(1));
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
