#![feature(globs)]
extern crate inotify;
extern crate time;

use colormap::{ColorMap, Color};
use cpubar::CpuBar;
use statusbar::StatusBar;

mod colormap;
mod cpubar;
mod statusbar;

fn main() {
    let mut cmap: ColorMap = ColorMap::new();
    cmap.add_pair(0,   Color::new(0., 0., 0.));
    cmap.add_pair(10,  Color::new(0.3, 0.3, 0.3));
    cmap.add_pair(30,  Color::new(0.7, 0.8, 0.25));
    cmap.add_pair(60,  Color::new(0.8, 0.6, 0.0));
    cmap.add_pair(100, Color::new(1.0, 0.0, 0.0));

    let mut cpu = CpuBar::new();
    cpu.initialize();

    cmap.test()
}
