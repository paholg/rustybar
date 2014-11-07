use std::fmt;
use statusbar::*;
use colormap::ColorMap;
use std::io::{BufferedReader, File};
use std::io::fs::PathExtensions;

static BGCOLOR: &'static str = "#000000";

/// A statusbar for battery information. All data is gathered from '/sys/class/power_supply/BAT#/'
pub struct BatteryBar {
    bat_num: uint,
    path_capacity: Path,
    path_status: Path,
    cmap: ColorMap,
    // the length, in pixels, of the displayed string.
    str_length: uint,
    bar: String,
    // the size of bars and spaces between them
    width: uint,
    space: uint,
    height: uint,
}

impl BatteryBar {
    pub fn new() -> BatteryBar {
        BatteryBar {
            bat_num: 0,
            path_capacity: Path::new("/"),
            path_status: Path::new("/"),
            cmap: ColorMap::new(),
            str_length: 0,
            bar: String::new(),
            width: 0,
            space: 0,
            height: 0,
        }
    }
    /// sets the battery number to use
    pub fn set_bat_num(&mut self, num: uint) {
        self.bat_num = num;
        let path = Path::new(format!("/sys/class/power_supply/BAT{}/", num).as_slice());
        if !path.exists() {
            panic!("The selected battery directory:\n\t{}\ndoes not exist and is needed for the battery bar. Perhaps you meant a different battery number or perhaps it isn't there. Go ahead and report this with the output of \"ls /sys/class/power_supply/", path.display());
        }
        self.path_capacity = Path::new(format!("/sys/class/power_supply/BAT{}/capacity", num).as_slice());
        if !self.path_capacity.exists() {
            panic!("The file:\n\t{}\ndoes not exist. It almost certainly should and is needed for the battery bar.", self.path_capacity.display());
        }
        self.path_status = Path::new(format!("/sys/class/power_supply/BAT{}/status", num).as_slice());
        if !self.path_status.exists() {
            panic!("The file:\n\t{}\ndoes not exist. It almost certainly should and is needed for the battery bar.", self.path_status.display());
        }
    }
}

impl StatusBar for BatteryBar {
    fn initialize(&mut self, width: uint, space: uint, height: uint) {
        self.width = width;
        self.space = space;
        self.height = height;
        // In case this hasn't been run. It doesn't hurt to run it twice.
        let num = self.bat_num;
        self.set_bat_num(num);
        // and run update() to finish up
        self.update();
    }
    fn update(&mut self) {
        let cap_string = File::open(&self.path_capacity).read_to_string().unwrap();
        let capacity: u8 = from_str(cap_string.trim().as_slice()).unwrap();
        let status_string = File::open(&self.path_status).read_to_string().unwrap();
        let status = match status_string.trim().as_slice() {
            "Charging" => "^fg(#00ff00) +",
            "Discharging" => "^fg(#ff0000) -",
            "Full" => "  ",
            _ => " ^bg(#ff0000)^fg(000000)*^bg()",
        };
        //println!("{}", status_string);
        self.bar.clear();
        self.bar.push_str(format!("^fg({})", self.cmap.map(capacity)).as_slice());
        // fixme: change when add_bar takes uints
        self.bar.add_bar((capacity as f64)/100., self.width, self.height);
        self.bar.push_str(status);
    }
    fn set_colormap(&mut self, cmap: Box<ColorMap>) {
        self.cmap = *cmap;
    }
    fn len(&self) -> uint {
        0
    }
}

impl fmt::Show for BatteryBar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.bar)
    }
}
