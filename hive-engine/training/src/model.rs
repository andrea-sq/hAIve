use crate::dataset::BoardBatch;
use burn::nn::loss::MseLoss;
use burn::nn::loss::Reduction::Mean;
use burn::nn::{Dropout, DropoutConfig};
use burn::tensor::activation::sigmoid;
use burn::tensor::backend::AutodiffBackend;
use burn::train::{InferenceStep, RegressionOutput, TrainOutput, TrainStep};
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
    dropout: Dropout,
}

#[derive(Config, Debug)]
pub struct ModelConfig {
    #[config(default = 512)]
    hidden_size_1: usize,
    #[config(default = 32)]
    hidden_size_2: usize,
    #[config(default = 1)]
    output_size: usize,
    #[config(default = 0.4)]
    dropout: f64,
}

impl ModelConfig {
    /// Returns the initialized model.
    pub fn init<B: Backend>(&self, device: &B::Device) -> Model<B> {
        Model {
            features_layer: LinearConfig::new(NNUE_BOARD, self.hidden_size_1)
                .with_bias(true)
                .init(device),
            // l1_layer: LinearConfig::new(self.hidden_size_1, self.hidden_size_2)
            //     .with_bias(true)
            //     .init(device),
            l1_layer: LinearConfig::new(self.hidden_size_1 * 2, self.hidden_size_2)
                .with_bias(true)
                .init(device),
            l2_layer: LinearConfig::new(self.hidden_size_2, self.output_size)
                .with_bias(true)
                .init(device),
            dropout: DropoutConfig::new(self.dropout).init(),
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
        let bw_tensor = Tensor::cat(vec![b.clone(), w.clone()], 1);
        let mask = stm
            .clone()
            .unsqueeze::<2>()
            .transpose()
            .expand(wb_tensor.shape());

        let accumulator =
            wb_tensor.mask_fill(mask.clone().bool_not(), 0) + bw_tensor.mask_fill(mask, 0);
        // let accumulator = wb_tensor;
        // Clamp implementation
        let l1_x = self.dropout.forward(accumulator).clamp(0.0, 1.0);
        let l2_x = self
            .dropout
            .forward(self.l1_layer.forward(l1_x))
            .clamp(0.0, 1.0);
        self.l2_layer.forward(l2_x)
        // let l1_x = w.clamp(0.0, 1.0);

        // let l2_x = self
        //     .dropout
        //     .forward(self.l1_layer.forward(l1_x))
        //     .clamp(0.0, 1.0);
        // let l3_x = self
        //     .dropout
        //     .forward(self.l2_layer.forward(l2_x))
        //     .clamp(0.0, 1.0);

        // self.l3_layer.forward(l3_x)
    }

    pub fn forward_step(&self, item: BoardBatch<B>) -> RegressionOutput<B> {
        let output = self.forward(item.white_features, item.black_features, item.stm);
        let targets = item.targets.unsqueeze_dim(1);
        let scaling = 40;
        // let offset = 20;
        // let s = (targets.clone() - offset) / scaling;
        // let sm = (-targets.clone() - offset) / scaling;
        // let sf = 0.5 * (1.0 + sigmoid(s) - sigmoid(sm));
        //
        // let q = (output.clone() - offset) / scaling;
        // let qm = (-output.clone() - offset) / scaling;
        // let qf = 0.5 * (1.0 + sigmoid(q) - sigmoid(qm));

        let l_targets = sigmoid(targets.clone() / scaling);

        let l_output = sigmoid(output.clone() / scaling);
        // let l_output = output.clone();
        // let epsilons = 1e-12;
        // let loss_cross: Tensor<B, 2> = (targets.clone() * (targets.clone() + epsilons).log()
        //     + (1.0 - targets.clone()) * (1.0 + epsilons)
        //     - targets.clone().log())
        //     - (output.clone() * (output.clone() + epsilons).log()
        //         + (1.0 - output.clone()) * (1.0 + epsilons)
        //         - output.clone().log());
        // let loss_cross: Tensor<B, 1> = loss_cross.mean_dim(1).squeeze_dim(1);
        let loss = MseLoss::new().forward(l_output.clone(), l_targets.clone(), Mean);
        // let loss = Tensor::abs(sf - qf)
        //     .powf_scalar(2.5)
        //     .mean_dim(1)
        //     .squeeze_dim(1);

        RegressionOutput {
            loss,
            output: l_output,
            targets: l_targets,
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
