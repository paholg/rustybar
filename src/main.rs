#![feature(globs)]
#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate inotify;
extern crate time;
extern crate core;

use std::io::{timer, Command};
use std::time::Duration;

use colormap::{ColorMap, Color};
use statusbar::StatusBar;
use cpubar::CpuBar;
use batterybar::BatteryBar;
use std::io::{File};
use std::io::fs::PathExtensions;


mod colormap;
mod statusbar;
mod cpubar;
mod batterybar;

fn get_resolution() -> uint {
    let mut process = match Command::new("xrandr").spawn() {
        Ok(out) => out,
        Err(e) => panic!("Failed to run xrandr with error: {}", e),
    };
    // fixme: replace unwrap()s with error checking
    let out = process.stdout.as_mut().unwrap().read_to_string().unwrap();
    let re = regex!(r"current\s(\d+)\sx\s\d+");
    let cap = re.captures_iter(out.as_slice()).nth(0).unwrap();
    let res: uint = from_str(cap.at(1)).unwrap();
    res
}

fn get_dpi(default: uint) -> uint {
    let path = Path::new("/var/log/Xorg.0.log");
    if !path.exists() {
        // fixme: make this an actual warning if such exists -- couldn't find it
        println!("Warning!! couldn't find Xorg log to set dpi. Using the default of {}. If this is wrong, your bars won't be sized correctly. The log file searched for was: {}", default, path.display());
        return default;
    }
    let x_log = File::open(&path).read_to_string().unwrap();
    let re = regex!(r"DPI.*?\((\d+),\s\d+\)");
    let mut dpi: uint = 0;
    for cap in re.captures_iter(x_log.as_slice()) {
        dpi = from_str(cap.at(1)).unwrap();
    }
    if dpi == 0 {
        println!("Warning!! I couldn't find your dpi. Using the default of {}. If this is wrong, your bars won't be sized correctly. The log file searched was: {}", default, path.display());
        return default;
    }
    dpi
}

fn main() {
    let mut cpu_cmap = box ColorMap::new();
    cpu_cmap.add_pair(0,   Color::new(0., 0., 0.));
    cpu_cmap.add_pair(10,  Color::new(0.3, 0.3, 0.3));
    cpu_cmap.add_pair(30,  Color::new(0.7, 0.8, 0.25));
    cpu_cmap.add_pair(60,  Color::new(0.8, 0.6, 0.0));
    cpu_cmap.add_pair(100, Color::new(1.0, 0.0, 0.0));
    cpu_cmap.test();

    let mut battery_cmap = box ColorMap::new();
    battery_cmap.add_pair(0,   Color::new(1., 0., 0.));
    battery_cmap.add_pair(100, Color::new(0., 1., 0.));

    let mut cpu = box CpuBar::new();
    cpu.set_colormap(cpu_cmap);
    let mut battery = box BatteryBar::new();
    battery.set_colormap(battery_cmap);

    let mut bars: Vec<Box<StatusBar>> = Vec::new();
    bars.push(battery);
    bars.push(cpu);
    for i in range(0u, bars.len()) {
        bars[i].initialize(25, 8, 11);
    }
    let dpi = get_dpi(96);
    let mut res = get_resolution();
    let font_size = 9u;
    // fixme: make sure this holds when it's not an even number.
    let width = font_size*dpi/72;

    for _ in range(0u, 1000) {
        timer::sleep(Duration::seconds(1));
        let new_res = get_resolution();
        if new_res != res {
            res = new_res;
            // fixme: update resolution here
        }
        for i in range(0u, bars.len()) {
            bars[i].update();
            print!("{}", bars[i]);
        }
        println!("");
    }
}
