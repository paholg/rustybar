use crate::color::Color;

/// A map that stores (value, color) pairs, which can be used to interpolate between
/// colors for arbitrary values. Values are in the range [0,100].
///
/// # Example
/// fixme: add example
#[derive(Clone, Debug)]
pub struct ColorMap {
    colors: Vec<Color>,
    values: Vec<f32>,
}

impl<'a> std::iter::FromIterator<&'a (f32, Color)> for ColorMap {
    fn from_iter<I: IntoIterator<Item = &'a (f32, Color)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut values = vec![];
        let mut colors = vec![];
        for &(v, c) in iter {
            values.push(v);
            colors.push(c);
        }
        assert!(
            values.len() > 1,
            "Must have at least two elements for a color_map"
        );

        ColorMap { values, colors }
    }
}

impl ColorMap {
    /// This does the interpolation, and gives you the color corresponding to the value
    /// called with, as dicated by the color map.
    pub fn map(&self, val: f32) -> Color {
        if val < self.values[0] {
            return self.colors[0];
        } else if val > self.values[self.values.len() - 1] {
            return self.colors[self.colors.len() - 1];
        }

        let mut i = 1;
        while self.values[i] < val && i < self.values.len() - 1 {
            i += 1;
        }
        let lower: f32 = (self.values[i] - val) / (self.values[i] - self.values[i - 1]);
        let upper: f32 = (val - self.values[i - 1]) / (self.values[i] - self.values[i - 1]);

        let interpolate = |c1, c2| (lower * (f32::from(c1)) + upper * f32::from(c2)) as u8;
        let red: u8 = interpolate(self.colors[i - 1].r, self.colors[i].r);
        let green: u8 = interpolate(self.colors[i - 1].g, self.colors[i].g);
        let blue: u8 = interpolate(self.colors[i - 1].b, self.colors[i].b);

        Color {
            r: red,
            g: green,
            b: blue,
        }
    }
}
