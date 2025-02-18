struct Unit<'a> {
    value: f32,
    name: &'a str,
}

impl Unit<'_> {
    const fn new(value: f32, name: &str) -> Unit {
        Unit { value, name }
    }
}

const K: f32 = 1024.0;
const M: f32 = 1024.0 * K;
const G: f32 = 1024.0 * M;
const T: f32 = 1024.0 * G;
const P: f32 = 1024.0 * T;
const E: f32 = 1024.0 * P;

const BYTE_UNITS: [Unit; 6] = [
    Unit::new(K, "k"),
    Unit::new(M, "M"),
    Unit::new(G, "G"),
    Unit::new(T, "T"),
    Unit::new(P, "P"),
    Unit::new(E, "E"),
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
