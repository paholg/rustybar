#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate inotify;
extern crate time;

use std::io::timer;
use std::time::Duration;
use colormap::{ColorMap, Color};
use statusbar::StatusBar;
use cpubar::CpuBar;
//use batterybar::BatteryBar;

mod colormap;
mod statusbar;
mod cpubar;
//mod batterybar;

fn main() {
    let mut cpu_cmap = box ColorMap::new();
    cpu_cmap.add_pair(0,   Color::new(1., 1., 1.));
    cpu_cmap.add_pair(10,  Color::new(0.3, 0.3, 0.3));
    cpu_cmap.add_pair(30,  Color::new(0.7, 0.8, 0.25));
    cpu_cmap.add_pair(60,  Color::new(0.8, 0.6, 0.0));
    cpu_cmap.add_pair(100, Color::new(1.0, 0.0, 0.0));
    cpu_cmap.test();

    let mut cpu = box CpuBar::new();
    cpu.set_colormap(cpu_cmap);

    let mut v: Vec<Box<StatusBar>> = Vec::new();
    v.push(cpu);
    v[0].initialize(22, 5, 11);
    for _ in range(0u, 1000) {
        timer::sleep(Duration::seconds(1));
        v[0].update();
        println!("{}", v[0]);
    }
    // for bar in v.iter() {
    //     println!("{}", bar);
    // }
}
