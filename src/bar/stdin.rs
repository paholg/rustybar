use std::sync::Arc;

use async_trait::async_trait;

use crate::producer::SingleQueue;

/// A statusbar for stdin.
#[derive(Clone, Debug)]
pub struct Stdin {
    width: u32,
}

impl Stdin {
    pub async fn new(padding: u32) -> Box<Stdin> {
        Box::new(Stdin { width: padding })
    }
}

#[async_trait]
impl crate::bar::Bar for Stdin {
    type Data = String;

    fn width(&self) -> u32 {
        self.width
    }

    fn render(&self, data: &Self::Data) -> String {
        data.clone()
    }

    fn data_queue(&self) -> SingleQueue<Arc<Self::Data>> {
        crate::producer::STDIN.clone()
    }
}
