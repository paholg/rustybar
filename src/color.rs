use std::fmt;
use std::num::ParseIntError;

/// An RGB triplet
#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// creates a Color object, guaranteeing that the colors are acceptable values.
    pub fn new(red: u8, green: u8, blue: u8) -> Color {
        Color {
            r: red,
            g: green,
            b: blue,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    ParseIntError(ParseIntError),
    Other(String),
}

impl std::str::FromStr for Color {
    type Err = ParseError;

    /// expects a color in the format "#ffffff"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 7 || s.bytes().next() != Some(b'#') {
            return Err(ParseError::Other(s.into()));
        }

        let num = u32::from_str_radix(&s[1..7], 16).map_err(ParseError::ParseIntError)?;
        let red = num >> 16;
        let green = (num >> 8) & 255;
        let blue = num & 255;

        Ok(Color {
            r: red as u8,
            g: green as u8,
            b: blue as u8,
        })
    }
}

impl fmt::Display for Color {
    /// Color triplets are formatted in the form "#ffffff"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
