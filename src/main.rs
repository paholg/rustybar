use rustybar::{
    bar::{self, Bar},
    util::screen::get_screens,
    RustyBar,
};
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut screens = Vec::new();
    let mut bars;

    loop {
        let new_screens = get_screens();
        if new_screens != screens {
            screens = new_screens;
            bars = Vec::new();
            for screen in &screens {
                let mut bar = init_bar().await;
                bar.start(screen).await;
                bars.push(bar);
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn init_bar() -> RustyBar {
    let font = rustybar::Font::new("Monospace-12".into(), 12);
    let height = 24;
    let ch = font.width;

    // Spacemacs dark colors
    let bg1 = "#292b2e".into();
    let bg2 = "#1f2022".into();
    let bg4 = "#0a0814".into();
    let aqua = "#2d9574".into();
    let blue = "#4f97d7".into();
    let magenta = "#a31db1".into();
    let red = "#f2241f".into();

    {
        // TODO: Move all this to methods
        let mut config = rustybar::config::write().await;
        config.font = font;
        config.height = height;
        config.background = bg4;
    }

    rustybar::RustyBar::new(
        vec![bar::Stdin::new(100).await.start()],
        vec![
            bar::Temp::new(
                [(40.0, aqua), (60.0, blue), (80.0, magenta), (100.0, red)]
                    .iter()
                    .collect(),
                ch * 2,
            )
            .await
            .start(),
            bar::Cpu::new(
                [
                    (0.0, bg2),
                    (0.2, bg1),
                    (0.4, aqua),
                    (0.8, magenta),
                    (1.0, red),
                ]
                .iter()
                .collect(),
                40,
                80,
                16,
                ch,
                ch * 2,
            )
            .await
            .start(),
            bar::Memory::new([(1e9, red), (3e9, blue), (8e9, aqua)].iter().collect(), 0)
                .await
                .start(),
        ],
        vec![
            bar::Network::new(
                [
                    (0.0, bg2),
                    (1e3, bg1),
                    (10e3, aqua),
                    (100e3, blue),
                    (1e6, magenta),
                    (50e6, red),
                ]
                .iter()
                .collect(),
                4 * ch,
            )
            .await
            .start(),
            bar::Battery::new(
                [(0.0, red), (0.3, magenta), (0.7, blue), (1.0, aqua)]
                    .iter()
                    .collect(),
                bar::BatteryColors {
                    charge: aqua,
                    discharge: red,
                    unknown: magenta,
                },
                40,
                16,
                ch,
                ch,
                ch * 2,
            )
            .await
            .start(),
            bar::Clock::new(blue, "%a %Y-%m-%d", 14, 2 * ch)
                .await
                .start(),
            bar::Clock::new(aqua, "%H:%M:%S", 8, ch).await.start(),
        ],
    )
}
