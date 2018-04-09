use bar::*;

use failure;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub font: String,
    pub left_gap: u32,
    pub right_gap: u32,
    pub height: u32,
    pub background: String,

    #[serde(default)]
    pub left: Vec<EntryConfig>,
    #[serde(default)]
    pub center: Vec<EntryConfig>,
    #[serde(default)]
    pub right: Vec<EntryConfig>,
}

#[derive(Debug, Deserialize)]
pub struct Space {
    space: i32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum EntryConfig {
    Bar(BarConfig),
    Space(Space),
}

enum Entry {
    Bar(Box<StatusBar>),
    Space(i32),
}

// Don't use space_available for the center bar
pub fn entries_to_bars(
    entry_configs: &[EntryConfig],
    char_width: u32,
    space_available: Option<u32>,
) -> Result<Vec<Box<StatusBar>>, failure::Error> {
    let mut entries: Vec<Entry> = {
        let entries: Result<Vec<_>, failure::Error> = entry_configs
            .iter()
            .map(|e| match e {
                &EntryConfig::Space(ref s) => Ok(Entry::Space(s.space)),
                &EntryConfig::Bar(ref b) => Ok(Entry::Bar(b.into_bar(char_width)?)),
            })
            .collect();
        entries?
    };

    let space_used: u32 = entries
        .iter()
        .map(|e| match e {
            &Entry::Space(s) => if s > 0 {
                s as u32
            } else {
                0
            },
            &Entry::Bar(ref b) => b.len(),
        })
        .sum();

    if let Some(space_available) = space_available {
        if space_used > space_available {
            bail!(
                "Out of space. {} used, but {} available.",
                space_used,
                space_available
            );
        }
    }

    let space_remaining = space_available.map(|a| a - space_used).unwrap_or(0);

    let dynamic_space_denom: u32 = entries
        .iter()
        .filter_map(|e| {
            if let &Entry::Space(s) = e {
                if s < 0 {
                    Some(-s as u32)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .sum();

    let dynamic_space_remainder = if dynamic_space_denom == 0 {
        0
    } else {
        space_remaining - ((space_remaining) / dynamic_space_denom * dynamic_space_denom)
    };

    if space_available.is_none() && dynamic_space_denom > 0 {
        bail!("Cannot use dynamic space on the center bar.");
    }

    let mut space = 0;
    for entry in entries.iter_mut() {
        match entry {
            &mut Entry::Space(s) => {
                let sp = if s < 0 {
                    ((-s) as u32 * space_remaining) / dynamic_space_denom
                } else {
                    s as u32
                };
                space += sp;
            }
            &mut Entry::Bar(ref mut bar) => {
                bar.set_lspace(space);
                space = 0;
            }
        }
    }

    let mut res: Vec<Box<StatusBar>> = entries
        .into_iter()
        .filter_map(|e| if let Entry::Bar(b) = e { Some(b) } else { None })
        .collect();

    if let Some(bar) = res.last_mut() {
        bar.set_rspace(space + dynamic_space_remainder);
    }

    Ok(res)
}
