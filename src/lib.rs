use consumer::{
    clock::{ClockConfig, ClockConsumer},
    cpu::{CpuConfig, CpuConsumer},
    Consumer, RegisterConsumer,
};
use enum_dispatch::enum_dispatch;
use futures::Stream;
use iced::Element;
use iced_layershell::to_layer_message;
use producer::{
    tick::{TickMessage, TickProducer},
    Producer, ProducerMap,
};
use serde::Deserialize;
use strum::EnumDiscriminants;

pub mod config;
pub mod consumer;
pub mod producer;
pub mod util;

#[enum_dispatch]
#[derive(EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum ProducerEnum {
    TickProducer,
}

#[enum_dispatch]
#[derive(Deserialize)]
pub enum ConsumerConfig {
    ClockConfig,
    CpuConfig,
}

#[enum_dispatch]
pub enum ConsumerEnum {
    ClockConsumer,
    CpuConsumer,
}

#[to_layer_message]
#[derive(Debug)]
pub enum Message {
    Tick(TickMessage),
}

impl Default for ProducerEnum {
    fn default() -> Self {
        todo!()
    }
}
