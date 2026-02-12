use crate::dataset::BoardBatch;
use burn::nn::loss::MseLoss;
use burn::nn::loss::Reduction::Mean;
use burn::tensor::backend::AutodiffBackend;
use burn::train::{InferenceStep, RegressionOutput, TrainOutput, TrainStep};
use burn::{
    nn::{Linear, LinearConfig},
    prelude::*,
};
use hive_library::{Board, NNUE_BOARD};

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    features_layer: Linear<B>,
    l1_layer: Linear<B>,
    l2_layer: Linear<B>,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    hidden_size_1: usize,
    hidden_size_2: usize,
    output_size: usize,
}

impl ModelConfig {
    /// Returns the initialized model.
    pub fn init<B: Backend>(&self, device: &B::Device) -> Model<B> {
        Model {
            features_layer: LinearConfig::new(NNUE_BOARD, self.hidden_size_1)
                .with_bias(true)
                .init(device),
            l1_layer: LinearConfig::new(self.hidden_size_1 * 2, self.hidden_size_2)
                .with_bias(true)
                .init(device),
            l2_layer: LinearConfig::new(self.hidden_size_2, self.output_size)
                .with_bias(true)
                .init(device),
        }
    }
}

impl<B: Backend> Model<B> {
    pub fn forward(
        &self,
        white_features: Tensor<B, 2, Bool>,
        black_features: Tensor<B, 2, Bool>,
        stm: Tensor<B, 1, Bool>,
    ) -> Tensor<B, 2> {
        let w = self.features_layer.forward(white_features.float());
        let b = self.features_layer.forward(black_features.float());
        let wb_tensor = Tensor::cat(vec![w.clone(), b.clone()], 1);
        let bw_tensor = Tensor::cat(vec![w.clone(), b.clone()], 1);
        // TODO: Check that unsqueeze_dim and expand is correct
        let mask = stm.clone().unsqueeze::<2>().expand(wb_tensor.shape());

        let accumulator =
            wb_tensor.mask_fill(mask.clone().bool_not(), 0) + bw_tensor.mask_fill(mask, 0);
        let l1_x = accumulator.clamp(0.0, 1.0);
        let l2_x = self.l1_layer.forward(l1_x).clamp(0.0, 1.0);
        self.l2_layer.forward(l2_x)
    }

    pub fn forward_step(&self, item: BoardBatch<B>) -> RegressionOutput<B> {
        let output = self.forward(item.white_features, item.black_features, item.stm);
        // TODO: Check that unsqueeze_dim is correct
        let targets = item.targets.unsqueeze_dim(1);
        let loss = MseLoss::new().forward(output.clone(), targets.clone(), Mean);
        RegressionOutput {
            loss,
            output,
            targets,
        }
    }
}

impl<B: AutodiffBackend> TrainStep for Model<B> {
    type Input = BoardBatch<B>;
    type Output = RegressionOutput<B>;

    fn step(&self, batch: BoardBatch<B>) -> TrainOutput<RegressionOutput<B>> {
        let item = self.forward_step(batch);

        TrainOutput::new(self, item.loss.backward(), item)
    }
}

impl<B: Backend> InferenceStep for Model<B> {
    type Input = BoardBatch<B>;
    type Output = RegressionOutput<B>;

    fn step(&self, batch: BoardBatch<B>) -> RegressionOutput<B> {
        self.forward_step(batch)
    }
}
