use burn::data::dataloader::batcher::Batcher;
use burn::data::dataset::{Dataset, InMemDataset};
use burn::prelude::*;
use burn::tensor::IndexingUpdateOp;
use hive_library::NNUE_BOARD;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardItem {
    white_features_indices: Vec<usize>,
    black_features_indices: Vec<usize>,
    stm: bool,
    score: f64,
}

#[derive(Clone, Default)]
pub struct BoardBatcher {}

#[derive(Clone, Debug)]
pub struct BoardBatch<B: Backend> {
    pub white_features: Tensor<B, 2, Bool>,
    pub black_features: Tensor<B, 2, Bool>,
    pub stm: Tensor<B, 1, Bool>,
    pub targets: Tensor<B, 1>,
}

pub struct BoardDataset {
    dataset: InMemDataset<BoardItem>,
}

impl Dataset<BoardItem> for BoardDataset {
    fn get(&self, index: usize) -> Option<BoardItem> {
        self.dataset.get(index)
    }

    fn len(&self) -> usize {
        self.dataset.len()
    }
}

impl BoardDataset {
    pub fn train() -> Result<Self, std::io::Error> {
        Self::new("train")
    }
    pub fn validation() -> Result<Self, std::io::Error> {
        Self::new("validation")
    }
    pub fn test() -> Result<Self, std::io::Error> {
        Self::new("test")
    }
    fn new(split: &str) -> Result<Self, std::io::Error> {
        let path_str = format!("dataset/{}.csv", split);
        let path = Path::new(&path_str);

        let mut rdr = csv::ReaderBuilder::new();
        let rdr = rdr.delimiter(b'\t');

        let dataset = InMemDataset::from_csv(path, rdr)?;
        let dataset = Self { dataset };

        Ok(dataset)
    }
}

impl<B: Backend> Batcher<B, BoardItem, BoardBatch<B>> for BoardBatcher {
    fn batch(&self, items: Vec<BoardItem>, device: &B::Device) -> BoardBatch<B> {
        let stm_t = Tensor::cat(
            items
                .iter()
                .map(|item| {
                    Tensor::<B, 1, Bool>::from_data([item.stm.elem::<B::BoolElem>()], device)
                })
                .collect::<Vec<_>>(),
            0,
        );
        let score_t = Tensor::cat(
            items
                .iter()
                .map(|item| Tensor::<B, 1>::from_data([item.score.elem::<B::FloatElem>()], device))
                .collect::<Vec<_>>(),
            0,
        );

        // TODO: Use sparse tensors
        let w_t = items
            .iter()
            .map(|indices| {
                let indices_t = Tensor::<B, 1, Int>::from_data(
                    indices.white_features_indices.as_slice(),
                    device,
                );
                let res_t = Tensor::<B, 1, Bool>::zeros(&[NNUE_BOARD], device);
                res_t.select_assign(
                    0,
                    indices_t.clone(),
                    indices_t.ones_like().bool(),
                    IndexingUpdateOp::Add,
                )
            })
            .collect::<Vec<_>>();
        let w_t = Tensor::stack::<2>(w_t, 0);
        // TODO: Use sparse tensors
        let b_t = items
            .iter()
            .map(|indices| {
                let indices_t = Tensor::<B, 1, Int>::from_data(
                    indices.black_features_indices.as_slice(),
                    device,
                );
                let res_t = Tensor::<B, 1, Bool>::zeros(&[NNUE_BOARD], device);
                res_t.select_assign(
                    0,
                    indices_t.clone(),
                    indices_t.ones_like().bool(),
                    IndexingUpdateOp::Add,
                )
            })
            .collect::<Vec<_>>();
        let b_t = Tensor::stack::<2>(b_t, 0);

        BoardBatch {
            white_features: w_t,
            black_features: b_t,
            stm: stm_t,
            targets: score_t,
        }
    }
}
