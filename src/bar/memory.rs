use crate::bar::{Bar, WriteBar, Writer};
use crate::colormap::{ColorMap, ColorMapConfig};
use failure;
use lazy_static;
use regex::Regex;
use std::{fs, io::Read, io::Write, path, process, thread, time};

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

impl Bar for Memory {
    fn len(&self) -> u32 {
        self.width
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        lazy_static! {
            static ref RE_TOT: Regex = Regex::new(r"MemTotal.*?(\d+)").unwrap();
            static ref RE_FREE: Regex = Regex::new(r"MemFree.*?(\d+)").unwrap();
            static ref RE_BUFFERS: Regex = Regex::new(r"Buffers.*?(\d+)").unwrap();
            static ref RE_CACHED: Regex = Regex::new(r"Cached.*?(\d+)").unwrap();
            static ref PATH: path::PathBuf = path::Path::new("/proc/meminfo").into();
        }

        let info = {
            let mut buffer = String::new();
            fs::File::open(&*PATH)?.read_to_string(&mut buffer)?;
            buffer
        };

        let free: f32 = RE_FREE
            .captures(&info)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse()?;
        let buffers: f32 = RE_BUFFERS
            .captures(&info)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse()?;
        let cached: f32 = RE_CACHED
            .captures(&info)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse()?;
        let total: f32 = RE_TOT
            .captures(&info)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse()?;
        let val = (total - free - buffers - cached) / total;

        w.bar(
            val,
            self.cmap.map((val * 100.) as u8),
            self.width,
            self.height,
        )?;
        w.space(self.rspace)?;
        w.write(b"\n")?;

        Ok(())
    }
}
