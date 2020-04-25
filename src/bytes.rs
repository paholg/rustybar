struct Unit<'a> {
    value: f32,
    name: &'a str,
}

impl<'a> Unit<'a> {
    const fn new(value: f32, name: &str) -> Unit {
        Unit { value, name }
    }
}

const BYTE_UNITS: [Unit; 6] = [
    Unit::new(1e3, "k"),
    Unit::new(1e6, "M"),
    Unit::new(1e9, "G"),
    Unit::new(1e12, "T"),
    Unit::new(1e15, "P"),
    Unit::new(1e18, "E"),
];

pub fn format_bytes(bytes: u64) -> String {
    let bytes = bytes as f32;
    let unit = BYTE_UNITS
        .iter()
        .take_while(|unit| unit.value <= bytes)
        .last()
        .unwrap_or(&BYTE_UNITS[0]);
    let value = bytes / unit.value;
    let n_decimals = if value < 10.0 {
        2
    } else if value < 100.0 {
        1
    } else {
        0
    };
    let decimal_point = if n_decimals == 0 { "." } else { "" };

    format!(
        "{:.*}{} {}",
        n_decimals,
        bytes / unit.value,
        decimal_point,
        unit.name
    )
}
