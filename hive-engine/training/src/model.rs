use burn::{
    nn::{
        Dropout, DropoutConfig, Linear, LinearConfig, Relu,
        conv::{Conv2d, Conv2dConfig},
        pool::{AdaptiveAvgPool2d, AdaptiveAvgPool2dConfig},
    },
    prelude::*,
};
use hive_library::NNUE_BOARD;

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    input_layer: Linear<B>,
    output_layer: Linear<B>,
    activation: Relu, //TODO: implement SCReLU
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    num_classes: usize,
    hidden_size: usize,
}

impl ModelConfig {
    /// Returns the initialized model.
    pub fn init<B: Backend>(&self, device: &B::Device) -> Model<B> {
        Model {
            activation: Relu::new(),
            input_layer: LinearConfig::new(NNUE_BOARD, self.hidden_size)
                .with_bias(true)
                .init(device),
            output_layer: LinearConfig::new(self.hidden_size, self.num_classes)
                .with_bias(true)
                .init(device),
        }
    }
}
