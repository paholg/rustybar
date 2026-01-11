pub mod niri;
pub mod tick;

// pub trait Producer {
//     fn produce(&mut self) -> BoxFuture<'_, Message>;

//     fn produce_stream(self: Box<Self>) -> Pin<Box<dyn Stream<Item = Message> + Send>>
//     where
//         Self: Sized + Send + 'static,
//     {
//         let mut this = self;
//         Box::pin(async_stream::stream! {
//             loop {
//                 yield this.produce().await;
//             }
//         })
//     }
// }

// #[derive(Default)]
// pub struct ProducerMap {
//     map: HashMap<&'static str, Box<dyn Producer>>,
// }

// impl ProducerMap {
//     pub fn register(&mut self, producer: impl Producer + 'static) {
//         self.map
//             .entry(producer.key())
//             .or_insert_with(|| broadcast::channel(1));
//     }

//     pub fn into_producers(self) -> Vec<Box<dyn Producer>> {
//         self.map.into_values().collect()
//     }
// }
