use burn::{
    nn::{Linear, LinearConfig},
    prelude::*,
};
use hive_library::NNUE_BOARD;

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    features_layer: Linear<B>,
    l1_layer: Linear<B>,
    l2_layer: Linear<B>,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    num_classes: usize,
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
        white_features: Tensor<B, 2>,
        black_features: Tensor<B, 2>,
        stm: bool,
    ) -> Tensor<B, 2> {
        let w = self.features_layer.forward(white_features);
        let b = self.features_layer.forward(black_features);
        let accumulator = (Tensor::cat(vec![w.clone(), b.clone()], 1) * stm)
            + (Tensor::cat(vec![b, w], 1) * !stm);
        let l1_x = accumulator.clamp(0.0, 1.0);
        let l2_x = self.l1_layer.forward(l1_x).clamp(0.0, 1.0);
        self.l2_layer.forward(l2_x)
    }
}
