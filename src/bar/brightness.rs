use colormap::{ColorMap, ColorMapConfig};

use inotify;
use failure;
use std::{fs, path, process, thread, time, io::Read, io::Write};

use bar::{write_one_bar, write_space, StatusBar};

#[derive(Debug, Deserialize)]
pub struct BrightnessConfig {
    #[serde(default)]
    colormap: ColorMapConfig,
    width: u32,
    height: u32,
}

/// A statusbar for brightness information. Uses information from /sys/class/backlight/
#[derive(Debug)]
pub struct Brightness {
    path_current: path::PathBuf,
    max: f32,
    cmap: ColorMap,
    width: u32,
    height: u32,
    lspace: u32,
    rspace: u32,
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
        // if dirs.cloned().count() > 1 {
        //     println!("You have multiple backlight directories. They are:");
        //     for dir in dirs {
        //         println!("\t{}", dir?.path().display());
        //     }
        //     println!("I am going to use the first one. To use another, edit the configuration file (Not yet enabled.).");
        // }
        let path = dirs.next().unwrap()?.path();
        let path_current = path.join("brightness");
        let path_max = path.join("max_brightness");

        let max_string = {
            let mut buffer = String::new();
            fs::File::open(&path_max)?.read_to_string(&mut buffer)?;
            buffer
        };

        let max: f32 = max_string.trim().parse()?;

        Ok(Brightness {
            path_current: path_current,
            max: max,
            cmap: ColorMap::from_config(&config.colormap)?,
            width: config.width,
            height: config.height,
            lspace: 0,
            rspace: 0,
        })
    }
}

impl StatusBar for Brightness {
    fn run(&self, w: &mut process::ChildStdin) -> Result<(), failure::Error> {
        let mut inotify = inotify::Inotify::init()?;
        inotify.add_watch(&self.path_current, inotify::WatchMask::MODIFY)?;

        let mut perform = || -> Result<(), failure::Error> {
            let current_string = {
                let mut buffer = String::new();
                fs::File::open(&self.path_current)?.read_to_string(&mut buffer)?;
                buffer
            };
            let current: f32 = current_string.trim().parse()?;
            let val = current / self.max;

            write_space(w, self.lspace)?;
            w.write(
                b"^ca(1,xdotool key XF86MonBrightnessUp)^ca(3,xdotool key XF86MonBrightnessDown)",
            )?;
            write_one_bar(
                w,
                val,
                self.cmap.map((val * 100.) as u8),
                self.width,
                self.height,
            )?;
            w.write(b"^ca()^ca()\n")?;
            write_space(w, self.rspace)?;
            Ok(())
        };

        let mut buffer = [0; 1024];
        loop {
            perform()?;
            inotify.read_events_blocking(&mut buffer)?.next();
        }

        Ok(())
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
