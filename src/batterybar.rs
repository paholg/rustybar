use statusbar::StatusBar;
use colormap::ColorMap;
use std::io::{BufferedReader, File};

/// A statusbar for battery information. All data is gathered from /proc/stat.
pub struct CpuBar {
    path: Path,
    idles: Vec<int>,
    #[allow(dead_code)]
    cmap: ColorMap,
    last: Timespec,
}

impl CpuBar {
    pub fn new() -> CpuBar {
        CpuBar {
            path: Path::new("/proc/stat"),
            idles: Vec::new(),
            cmap: ColorMap::new(),
            last: Timespec{sec: 0, nsec: 0},
        }
    }
}

impl StatusBar for CpuBar {
    fn initialize(&mut self) {
//        let mut info_file = BufferedReader::new(File::open(&Path::new("/proc/cpuinfo")));

        self.last = get_time();
        let mut file = BufferedReader::new(File::open(&self.path));
        for line in file.lines().skip(1) {
            let ln = line.unwrap();
            if ln.contains("cpu") {
                let val: Option<int> = from_str(ln.split(' ').skip(4).next().unwrap());
                let new_idle = val.unwrap();
                self.idles.push(new_idle);
            }
            else { break; }
        }
    }
    fn update(&mut self) {
    }
}
