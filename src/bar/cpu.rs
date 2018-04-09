use colormap::{ColorMap, ColorMapConfig};

use chrono;
use failure;
use regex;
use std::{fs, path, process, thread, time, io::Read, io::Write};

use bar::{write_one_bar, write_sep, write_space, StatusBar};

#[derive(Debug, Deserialize)]
pub struct CpuConfig {
    width: u32,
    height: u32,
    space: u32,
    colormap: ColorMapConfig,
}

/// A statusbar for cpu information. All data is gathered from /proc/stat and /proc/cpuinfo.
#[derive(Debug)]
pub struct Cpu {
    // Gives a mapping from the order the OS lists the processors to the order
    // we want to display them. We want to group hyperthreads on a single core,
    // but they tend not to be listed in the appropriate order.
    proc_map: Vec<u32>,
    cmap: ColorMap,
    procs_per_core: u32,
    num_cores: u32,
    width: u32,
    space: u32,
    height: u32,
    lspace: u32,
    rspace: u32,
}

impl Cpu {
    pub fn from_config(config: &CpuConfig, _char_width: u32) -> Result<Cpu, failure::Error> {
        let info_path = path::Path::new("/proc/cpuinfo");
        if !info_path.is_file() {
            bail!(
                "The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?",
                info_path.display()
            );
        }
        let cpu_info = {
            let mut buffer = String::new();
            fs::File::open(&info_path)?.read_to_string(&mut buffer)?;
            buffer
        };

        // want to first get number of cores and processors
        let re_core = regex::Regex::new(r"(?s)cpu\scores\s:\s(\d+).*?siblings\s:\s(\d+)")?;
        let cap = re_core.captures(&cpu_info).unwrap();

        let num_cores = cap.get(1).unwrap().as_str().parse()?;
        let num_procs: u32 = cap.get(2).unwrap().as_str().parse()?;
        if num_procs == 0 {
            bail!("Something went wrong with finding number of processors.");
        }
        let procs_per_core = num_procs / num_cores;

        let mut proc_map = vec![0; num_procs as usize];

        // Now we want to get the map from processor number to display order.
        // note: I have the .*? after core id because \s:\s doesn't match for
        // some reason.
        let re_map = regex::Regex::new(r"(?s)processor\s:\s(\d+).*?core id.*?(\d+)")?;
        let mut proc_counter = vec![0; num_cores as usize];

        for cap in re_map.captures_iter(&cpu_info) {
            let proc_id: u32 = cap.get(1).unwrap().as_str().parse()?;
            let core_id: u32 = cap.get(2).unwrap().as_str().parse()?;
            proc_map[proc_id as usize] = core_id * procs_per_core + proc_counter[core_id as usize];
            proc_counter[core_id as usize] += 1;
        }

        Ok(Cpu {
            proc_map: proc_map,
            cmap: ColorMap::from_config(&config.colormap)?,
            num_cores: num_cores,
            procs_per_core: procs_per_core,
            width: config.width,
            space: config.space,
            height: config.height,
            lspace: 0,
            rspace: 0,
        })
    }
}

impl StatusBar for Cpu {
    fn run(&self, w: &mut process::ChildStdin) -> Result<(), failure::Error> {
        let path = path::Path::new("/proc/stat");
        if !path.is_file() {
            bail!(
                "The file {} does not exist. You cannot use the cpu bar without it. Are you sure you're running GNU/Linux?",
                path.display()
            );
        }
        let re = regex::Regex::new(r"cpu(\d+)\s(\d+\s){3}(\d+)")?;
        let mut old_idles: Vec<u32> = vec![0; self.proc_map.len()];
        // setting old_idles (this is redundant code and should be removed for a better solution)
        let info = {
            let mut buffer = String::new();
            fs::File::open(&path)?.read_to_string(&mut buffer)?;
            buffer
        };
        for cap in re.captures_iter(&info) {
            let proc_id: usize = cap.get(1).unwrap().as_str().parse()?;
            let idle: u32 = cap.get(3).unwrap().as_str().parse()?;
            old_idles[self.proc_map[proc_id] as usize] = idle;
        }
        // -------------------
        let mut last = chrono::Local::now();

        loop {
            thread::sleep(time::Duration::from_secs(1));

            let mut idles: Vec<u32> = vec![0; self.proc_map.len()];

            let info = {
                let mut buffer = String::new();
                fs::File::open(&path)?.read_to_string(&mut buffer)?;
                buffer
            };
            for cap in re.captures_iter(&info) {
                let proc_id: usize = cap.get(1).unwrap().as_str().parse()?;
                let idle: u32 = cap.get(3).unwrap().as_str().parse()?;
                idles[self.proc_map[proc_id] as usize] = idle;
            }

            let now = chrono::Local::now();
            let dt = now.signed_duration_since(last);
            last = now;
            write_space(w, self.lspace)?;
            write_sep(w, 2 * self.height)?;
            write_space(w, self.space)?;
            for i in 0..idles.len() {
                let usage = 1.0
                    - (idles[i] as f32 - old_idles[i] as f32)
                        / (dt.num_nanoseconds().unwrap() as f32) * 1e7;
                let val = if usage > 1.0 {
                    1.0
                } else if usage < 0.0 {
                    0.0
                } else {
                    usage
                };
                let color = self.cmap.map((val * 100.0) as u8);
                write_one_bar(w, val, color, self.width, self.height)?;
                write_space(w, self.space)?;
                if i > 0 && (i as u32 - 1) % self.procs_per_core == 0 {
                    write_sep(w, 2 * self.height)?;
                    if i < idles.len() - 1 {
                        write_space(w, self.space)?;
                    }
                }
            }
            write_space(w, self.rspace)?;
            w.write(b"\n")?;
            old_idles = idles;
        }
    }

    fn len(&self) -> u32 {
        self.lspace + (self.width + self.space) * self.procs_per_core * self.num_cores
            + (self.space + 2) * (self.num_cores + 1) - self.space + self.rspace
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
