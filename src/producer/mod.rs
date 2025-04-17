use std::collections::HashMap;

use enum_dispatch::enum_dispatch;
use futures::Stream;

use crate::{Message, ProducerEnum, ProducerEnumDiscriminants};

pub mod tick;

#[enum_dispatch(ProducerEnum)]
#[allow(async_fn_in_trait)]
pub trait Producer: Default {
    async fn produce(&mut self) -> Message;

    fn produce_stream(self) -> impl Stream<Item = Message> {
        let mut this = self;
        async_stream::stream! {
            loop {
                yield this.produce().await;
            }
        }
    }
}

#[derive(Default)]
pub struct ProducerMap {
    map: HashMap<ProducerEnumDiscriminants, ProducerEnum>,
}

impl ProducerMap {
    pub fn register(&mut self, producer: ProducerEnum) {
        let key = (&producer).into();
        self.map.entry(key).or_insert(producer);
    }

    pub fn into_producers(self) -> Vec<ProducerEnum> {
        self.map.into_values().collect()
    }
}
