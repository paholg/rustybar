use std::fmt;
use statusbar::StatusBar;
use colormap::ColorMap;
use std::io::{BufferedReader, File};

/// A statusbar for battery information. All data is gathered from /proc/stat.
pub struct BatteryBar {
    path: Path,
    cmap: ColorMap,
}

impl BatteryBar {
    pub fn new() -> BatteryBar {
        BatteryBar {
            // fixme: real path
            path: Path::new("/"),
            cmap: ColorMap::new(),
        }
    }
}

impl StatusBar for BatteryBar {
    fn initialize(&mut self) {
    }
    fn update(&mut self) {
    }
    fn set_colormap(&mut self, cmap: &ColorMap) {
    }
    fn len(&self) -> uint {
        0
    }
}

impl fmt::Show for BatteryBar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}
