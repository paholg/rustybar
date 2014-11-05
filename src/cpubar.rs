extern crate time;

use std::fmt;
//use regex::Regex;
use statusbar::{StatusBar, FormatBar};
use colormap::ColorMap;
use std::io::{File};
use std::io::fs::PathExtensions;

use self::time::{Timespec, get_time};

static BGCOLOR: &'static str = "#000000";
static TEXTCOLOR: &'static str = "#888888";


/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
pub struct CpuBar {
    idles: Vec<uint>,
    // Gives a mapping from the order the OS lists the processors to the order we want
    // to display them. We want to group hyperthreads on a single core, but they tend
    // not to be listed in the appropriate order.
    proc_map: Vec<uint>,
    cmap: ColorMap,
    last: Timespec,
    procs_per_core: uint,
    num_cores: uint,
    // the length, in pixels, of the displayed string.
    str_length: uint,
    bar: String,
    // the size of bars and spaces between them
    width: uint,
    space: uint,
    height: uint,
}

impl CpuBar {
    pub fn new() -> CpuBar {
        CpuBar {
            idles: Vec::new(),
            proc_map: Vec::new(),
            cmap: ColorMap::new(),
            last: Timespec{sec: 0, nsec: 0},
            num_cores: 0,
            procs_per_core: 0,
            str_length: 0,
            bar: String::new(),
            width: 0,
            space: 0,
            height: 0,
        }
    }
}

impl StatusBar for CpuBar {
    fn initialize(&mut self, width: uint, space: uint, height: uint) {
        self.width = width;
        self.space = space;
        self.height = height;
        if self.idles.len() > 0 {
            panic!("Tried to initialize CpuBar, but it was already initialized.");
        }
        let info_path = Path::new("/proc/cpuinfo");
        if !info_path.is_file() {
            panic!("The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?", info_path.display());
        }
        let cpu_info = File::open(&info_path).read_to_string().unwrap();

        // want to first get number of cores and processors
        let re_core = regex!(r"(?s)cpu\scores\s:\s(\d+).*?siblings\s:\s(\d+)");
        let cap = re_core.captures_iter(cpu_info.as_slice()).nth(0).unwrap();
        self.num_cores = from_str(cap.at(1)).unwrap();
        let procs: uint = from_str(cap.at(2)).unwrap();
        if procs == 0 { panic!("Something went wrong with finding number of processors."); }
        self.procs_per_core = procs/self.num_cores;
        self.proc_map.grow(procs, 0);
        self.idles.grow(procs, 0);
        // now we want to get the map from processor number to display order
        // note: I have the .*? after core id because \s:\s doesn't match for some reason.
        let re_map = regex!(r"(?s)processor\s:\s(\d+).*?core id.*?(\d+)");
        let mut proc_counter: Vec<uint> = Vec::new();
        proc_counter.grow(self.num_cores, 0);
        for cap in re_map.captures_iter(cpu_info.as_slice()) {
            let proc_id: uint = from_str(cap.at(1)).unwrap();
            let core_id: uint = from_str(cap.at(2)).unwrap();
            self.proc_map[proc_id] = core_id*self.procs_per_core + proc_counter[core_id];
            proc_counter[core_id] += 1;
        }
        // now we'll get initial idle values
        let path = Path::new("/proc/stat");
        if !path.is_file() {
            panic!("The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?", path.display());
        }
        let info = File::open(&path).read_to_string().unwrap();
        let re = regex!(r"cpu(\d+)\s(\d+\s){3}(\d+)");
        for cap in re.captures_iter(info.as_slice()) {
            let proc_id: uint = from_str(cap.at(1)).unwrap();
            let idle: uint = from_str(cap.at(3)).unwrap();
            self.idles[self.proc_map[proc_id]] = idle;
        }
        self.last = get_time();
        // set length
        self.str_length = (self.width + self.space)*self.procs_per_core*self.num_cores +
            (self.space + 2)*(self.num_cores + 1) - self.space;
        // now we've finished the initialization, we'll run update to make sure everything is set
        self.update();
    }
    fn update(&mut self) {
        let path = Path::new("/proc/stat");
        if !path.is_file() {
            panic!("The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?", path.display());
        }
        // get the new idle values
        let info = File::open(&path).read_to_string().unwrap();
        let re = regex!(r"cpu(\d+)\s(\d+\s){3}(\d+)");
        let mut new_idles: Vec<uint> = Vec::new();
        new_idles.grow(self.idles.len(), 0);
        for cap in re.captures_iter(info.as_slice()) {
            let proc_id: uint = from_str(cap.at(1)).unwrap();
            let idle: uint = from_str(cap.at(3)).unwrap();
            new_idles[self.proc_map[proc_id]] = idle;
        }
        let now = get_time();
        let dt = now - self.last;
        // let dt: f64 = (dt_dur.num_nanoseconds().unwrap() as f64)*1e-9;
        self.last = now;
        let mut vals: Vec<uint> = Vec::new();
        vals.grow(self.idles.len(), 0);
        self.bar.clear();

        self.bar.push_str(format!("^bg({})^fg({})^r(2x{})^r({}x0)", BGCOLOR, TEXTCOLOR,
                                  2*self.height, self.space).as_slice());

        for i in range(0u, self.idles.len()) {
            let val = 1.0 - ((new_idles[i] - self.idles[i]) as f64)/(dt.num_nanoseconds().unwrap() as f64)*1e7;
            let color =  self.cmap.map((val*100.0) as u8);
            self.bar.push_str(format!("^fg({})", color).as_slice());
            self.bar.add_bar(val, self.width, self.height);
            self.bar.add_space(self.space);
            if (i-1) % self.procs_per_core == 0 {
                self.bar.push_str(format!("^fg({})^r(2x{})", TEXTCOLOR, 2*self.height).as_slice());
            }
            if i < self.idles.len()-1 {
                self.bar.add_space(self.space);
            }
        }
        self.idles = new_idles;
    }
    fn set_colormap(&mut self, cmap: Box<ColorMap>) {
        self.cmap = *cmap;
    }
    fn len(&self) -> uint {
        self.str_length
    }
}

impl fmt::Show for CpuBar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.bar)
    }
}
