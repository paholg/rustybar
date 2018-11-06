use crate::colormap::{ColorMap, ColorMapConfig};

use failure;
use inotify;
use std::{fs, io::Read, io::Write, path};

use crate::bar::{Bar, WriteBar, Writer};

#[derive(Debug, Deserialize)]
pub struct BrightnessConfig {
    #[serde(default)]
    colormap: ColorMapConfig,
    width: u32,
    height: u32,
}

use debug_stub_derive::DebugStub;
/// A statusbar for brightness information. Uses information from /sys/class/backlight/
#[derive(DebugStub)]
pub struct Brightness {
    path_current: path::PathBuf,
    max: f32,
    cmap: ColorMap,
    width: u32,
    height: u32,
    #[debug_stub = "Inotify"]
    inotify: inotify::Inotify,
    inotify_buffer: Vec<u8>,
}

impl Brightness {
    pub fn from_config(
        config: &BrightnessConfig,
        _char_width: u32,
    ) -> Result<Brightness, failure::Error> {
        let base_path = path::Path::new("/sys/class/backlight");

        if !base_path.is_dir() {
            bail!("The directory: {} doesn't exist. The brightness bar won't work.");
        }

        let mut dirs = fs::read_dir(&base_path)?;
        let dir = dirs.next();
        let next_dir = dirs.next();
        if next_dir.is_some() {
            warn!("You have multiple backlight directories. In {:?}. I am going to use the first one. To use another, edit the configuration file (Not yet enabled).", base_path);
        }
        if dir.is_none() {
            bail!(
                "No backlight directories found! I looked in {:?}",
                base_path
            );
        }
        let path = dir.unwrap()?.path();
        let path_current = path.join("brightness");
        let path_max = path.join("max_brightness");

        let max_string = {
            let mut buffer = String::new();
            fs::File::open(&path_max)?.read_to_string(&mut buffer)?;
            buffer
        };

        let max: f32 = max_string.trim().parse()?;

        let mut inotify = inotify::Inotify::init()?;
        inotify.add_watch(&path_current, inotify::WatchMask::MODIFY)?;

        Ok(Brightness {
            path_current,
            max,
            cmap: ColorMap::from_config(&config.colormap)?,
            width: config.width,
            height: config.height,
            inotify,
            inotify_buffer: vec![0; 1024],
        })
    }
}

impl Bar for Brightness {
    fn len(&self) -> u32 {
        self.width
    }

    fn write(&mut self, w: &mut Writer) -> Result<(), failure::Error> {
        let current_string = {
            let mut buffer = String::new();
            fs::File::open(&self.path_current)?.read_to_string(&mut buffer)?;
            buffer
        };
        let current: f32 = current_string.trim().parse()?;
        let val = current / self.max;

        w.write_all(
            b"^ca(1,xdotool key XF86MonBrightnessUp)^ca(3,xdotool key XF86MonBrightnessDown)",
        )?;

        w.bar(
            val,
            self.cmap.map((val * 100.) as u8),
            self.width,
            self.height,
        )?;
        w.write_all(b"^ca()^ca()")?;
        Ok(())
    }

    fn block(&mut self) -> Result<(), failure::Error> {
        self.inotify
            .read_events_blocking(&mut self.inotify_buffer)?;
        Ok(())
    }
}
