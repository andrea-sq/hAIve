use crate::dataset::{BoardBatcher, BoardDataset};
use crate::model::ModelConfig;
use burn::config::Config;
use burn::data::dataloader::DataLoaderBuilder;
use burn::optim::AdamConfig;
use burn::prelude::*;
use burn::record::CompactRecorder;
use burn::tensor::backend::AutodiffBackend;
use burn::train::metric::LossMetric;
use burn::train::{Learner, SupervisedTraining};

// #[derive(Config, Debug)]
// pub struct ExponentialLrSchedulerConfig {
//     // The initial learning rate.
//     #[config(default = 1.0e-5)]
//     initial_lr: LearningRate,
//     // The constant that the learning rate is multiplied by on each iteration.
//     #[config(default = 0.992)]
//     gamma: f64,
// }
//
// impl ExponentialLrSchedulerConfig {
//     /// Initializes a [exponential learning rate scheduler](ExponentialLrScheduler).
//     ///
//     /// # Errors
//     ///
//     /// An error will be returned if any of the following conditions is true:
//     ///
//     /// * `initial_lr` is out of range (0.0, 1.0]
//     /// * `gamma` is out of range (0.0, 1.0]
//     pub fn init(&self) -> ExponentialLrScheduler {
//         if self.initial_lr <= 0. || self.initial_lr > 1. {
//             panic!("Initial learning rate must be greater than 0 and at most 1");
//         }
//         if self.gamma <= 0. || self.gamma > 1. {
//             panic!("Gamma must be greater than 0 and at most 1");
//         }
//
//         ExponentialLrScheduler {
//             // Such an initial value eliminates the need for special-case handling of the first
//             // learning rate.
//             previous_lr: self.initial_lr / self.gamma,
//             gamma: self.gamma,
//         }
//     }
// }
//
// /// A exponential learning rate scheduler.
// ///
// /// See [ExponentialLrSchedulerConfig] for more information.
// #[derive(Clone, Copy, Debug)]
// pub struct ExponentialLrScheduler {
//     // The previous iteration's learning rate.
//     previous_lr: LearningRate,
//     // The constant that the learning rate is multiplied by on each iteration.
//     gamma: f64,
// }
//
// impl LrScheduler for ExponentialLrScheduler {
//     type Record<B: Backend> = LearningRate;
//
//     fn step(&mut self) -> LearningRate {
//         self.previous_lr *= self.gamma;
//         self.previous_lr
//     }
//
//     fn to_record<B: Backend>(&self) -> Self::Record<B> {
//         self.previous_lr
//     }
//
//     fn load_record<B: Backend>(mut self, record: Self::Record<B>) -> Self {
//         self.previous_lr = record;
//         self
//     }
// }

#[derive(Config, Debug)]
pub struct TrainingConfig {
    pub model: ModelConfig,
    pub optimizer: AdamConfig,
    #[config(default = 800)]
    pub num_epochs: usize,
    #[config(default = 256)]
    pub batch_size: usize,
    #[config(default = 1)]
    pub num_workers: usize,
    #[config(default = 1337)]
    pub seed: u64,
    #[config(default = 1.0e-5)]
    pub learning_rate: f64,
}

fn create_artifact_dir(artifact_dir: &str) {
    // Remove existing artifacts before to get an accurate learner summary
    std::fs::remove_dir_all(artifact_dir).ok();
    std::fs::create_dir_all(artifact_dir).ok();
}

pub fn train<B: AutodiffBackend>(artifact_dir: &str, config: TrainingConfig, device: B::Device) {
    create_artifact_dir(artifact_dir);
    config
        .save(format!("{artifact_dir}/config.json"))
        .expect("Config should be saved successfully");

    let model = config.model.init::<B>(&device);
    B::seed(&device, config.seed);

    let batcher = BoardBatcher::default();

    let dataloader_train = DataLoaderBuilder::new(batcher.clone())
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(BoardDataset::train().unwrap());

    let dataloader_valid = DataLoaderBuilder::new(batcher)
        .batch_size(config.batch_size)
        .shuffle(config.seed)
        .num_workers(config.num_workers)
        .build(BoardDataset::validation().unwrap());

    let training = SupervisedTraining::new(artifact_dir, dataloader_train, dataloader_valid)
        .metric_train_numeric(LossMetric::new())
        .metric_valid_numeric(LossMetric::new())
        .with_file_checkpointer(CompactRecorder::new())
        .num_epochs(config.num_epochs)
        .summary();

    let learner = Learner::new(model, config.optimizer.init(), config.learning_rate);

    let result = training.launch(learner);

    result
        .model
        .save_file(format!("{artifact_dir}/model"), &CompactRecorder::new())
        .expect("Trained model should be saved successfully");
}
