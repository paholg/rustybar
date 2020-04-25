use std::process;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Screen {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
}

pub fn get_screens() -> Vec<Screen> {
    let xrandr = process::Command::new("xrandr")
        .arg("--listactivemonitors")
        .output()
        .expect("xrandr command failed")
        .stdout;
    let out = String::from_utf8(xrandr).expect("xrandr should always output utf8");

    crate::regex!(r"(\d+):.* (\d+)/\d+x(\d+)/\d+\+(\d+)\+(\d+)")
        .captures_iter(&out)
        .map(|cap| Screen {
            id: cap[1].parse().unwrap(),
            width: cap[2].parse().unwrap(),
            // height : cap[3].parse().unwrap(),
            x: cap[4].parse().unwrap(),
            y: cap[5].parse().unwrap(),
        })
        .collect()
}
