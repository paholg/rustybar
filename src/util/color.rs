use iced::Color;
use serde::{Deserialize, Serialize};

/// A map that stores (value, color) pairs, which can be used to interpolate between
/// colors for arbitrary values.
///
/// # Example
/// fixme: add example
#[derive(Clone, Debug)]
pub struct Colormap {
    colors: Vec<Color>,
    values: Vec<f32>,
}

impl<'de> Deserialize<'de> for Colormap {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

impl Serialize for Colormap {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'a> std::iter::FromIterator<&'a (f32, Color)> for Colormap {
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

        Colormap { values, colors }
    }
}

impl Colormap {
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

        let interpolate = |c1, c2| lower * (c1) + upper * c2;
        let red = interpolate(self.colors[i - 1].r, self.colors[i].r);
        let green = interpolate(self.colors[i - 1].g, self.colors[i].g);
        let blue = interpolate(self.colors[i - 1].b, self.colors[i].b);
        let alpha = interpolate(self.colors[i - 1].a, self.colors[i].a);

        Color {
            r: red,
            g: green,
            b: blue,
            a: alpha,
        }
    }
}
