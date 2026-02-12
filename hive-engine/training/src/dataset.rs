use burn::data::dataloader::batcher::Batcher;
use burn::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardItem {}

#[derive(Clone, Default)]
pub struct BoardBatcher {}

#[derive(Clone, Debug)]
pub struct BoardBatch<B: Backend> {
    pub white_features: Tensor<B, 2, Int>,
    pub black_features: Tensor<B, 2, Int>,
    pub stm: Tensor<B, 1, Bool>,
    pub targets: Tensor<B, 1>,
}

impl<B: Backend> Batcher<B, BoardItem, BoardBatch<B>> for BoardBatcher {
    fn batch(&self, items: Vec<BoardItem>, device: &B::Device) -> BoardBatch<B> {
        todo!()
    }
}
