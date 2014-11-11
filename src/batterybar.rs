//extern crate libc;

use statusbar::*;
use colormap::ColorMap;
use std::io::File;
use std::io::fs::PathExtensions;
use std::io::timer;
use std::time::Duration;
use std::io::pipe;


/// A statusbar for battery information. All data is gathered from '/sys/class/power_supply/BAT#/'
pub struct BatteryBar {
    bat_num: uint,
    path_capacity: Path,
    path_status: Path,
    cmap: ColorMap,
    // the size of bars and spaces between them
    char_width: uint,
    pub width: uint,
    pub space: uint,
    pub height: uint,
}

impl BatteryBar {
    pub fn new() -> BatteryBar {
        BatteryBar {
            bat_num: 0,
            path_capacity: Path::new("/"),
            path_status: Path::new("/"),
            cmap: ColorMap::new(),
            char_width: 0,
            width: 30,
            space: 8,
            height: 10,
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
    fn initialize(&mut self, char_width: uint) {
        self.char_width = char_width;
        // In case set_bat_num() hasn't been run. It doesn't hurt to run it twice.
        let num = self.bat_num;
        self.set_bat_num(num);
    }
    fn run(&self, mut stream: Box<pipe::PipeStream>) {
        loop {
            let cap_string = File::open(&self.path_capacity).read_to_string().unwrap();
            let capacity: u8 = from_str(cap_string.trim().as_slice()).unwrap();

            write_one_bar(&mut *stream, (capacity as f32)/100., self.cmap.map(capacity), self.width, self.height);
            write_space(&mut *stream, self.space);
            let status_string = File::open(&self.path_status).read_to_string().unwrap();
            let status = match status_string.trim().as_slice() {
                "Charging" => "^fg(#00ff00)+\n",
                "Discharging" => "^fg(#ff0000)-\n",
                "Full" => " \n",
                _ => "^bg(#ff0000)^fg(#000000)*^bg()\n",
            };
            match stream.write_str(status) {
                Err(msg) => println!("Trouble writing to battery bar: {}", msg),
                Ok(_) => (),
            }
            timer::sleep(Duration::seconds(1));
        }
    }
    fn set_colormap(&mut self, cmap: Box<ColorMap>) {
        self.cmap = *cmap;
    }
    fn len(&self) -> uint {
        self.width + self.space + self.char_width
    }
}
