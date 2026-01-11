use std::sync::LazyLock;

use crate::{
    config::{GlobalConfig, RustybarConfig},
    consumer::Consumer,
};

pub mod config;
pub mod consumer;
pub mod iced_bar;
pub mod producer;
pub mod util;

pub static APP: LazyLock<Rustybar> = LazyLock::new(|| {
    let config = RustybarConfig::default();
    build(config)
});

pub struct Rustybar {
    config: GlobalConfig,

    left: Vec<Box<dyn Consumer>>,
    center: Vec<Box<dyn Consumer>>,
    right: Vec<Box<dyn Consumer>>,
}

fn build(config: RustybarConfig) -> Rustybar {
    let left = config.left.into_iter().map(|c| c.into_consumer()).collect();
    let center = config
        .center
        .into_iter()
        .map(|c| c.into_consumer())
        .collect();
    let right = config
        .right
        .into_iter()
        .map(|c| c.into_consumer())
        .collect();

    Rustybar {
        config: config.global,
        left,
        center,
        right,
    }
}
