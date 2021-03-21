pub fn bar(val: f32, color: super::color::Color, width: u32, height: u32) -> String {
    let wfill = (val * (width as f32) + 0.5) as u32;
    let wempty = width - wfill;
    format!(
        "^fg({})^r({2}x{1})^ro({3}x{1})",
        color, height, wfill, wempty
    )
}

pub fn space(width: u32) -> String {
    format!("^r({}x0)", width)
}

// TODO
// pub fn sep(height: u32) -> String {
//     format!("^fg({})^r(2x{})", TEXTCOLOR, height)
// }
