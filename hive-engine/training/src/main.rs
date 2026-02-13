#![recursion_limit = "256"]
mod dataset;
mod model;
mod training;

// use crate::training::ExponentialLrSchedulerConfig;
use crate::{model::ModelConfig, training::TrainingConfig};
use burn::backend::{Autodiff, Wgpu};
use burn::optim::AdamConfig;

fn main() {
    type MyBackend = Wgpu<f32, i32>;
    type MyAutodiffBackend = Autodiff<MyBackend>;

    let device = burn::backend::wgpu::WgpuDevice::default();
    let artifact_dir = "./haive_training_results";
    training::train::<MyAutodiffBackend>(
        artifact_dir,
        TrainingConfig::new(
            ModelConfig::new(256, 32, 1),
            AdamConfig::new(),
            // ExponentialLrSchedulerConfig::new(),
        ),
        device.clone(),
    );
    // let config = TrainingConfig::new(
    //     ModelConfig::new(256, 32, 1),
    //     AdamConfig::new(),
    //     ExponentialLrSchedulerConfig::new(),
    // );
    //
    // let batcher = BoardBatcher::default();
    //
    // let dataloader_train = DataLoaderBuilder::<MyBackend, _, BoardBatch<_>>::new(batcher.clone())
    //     .batch_size(config.batch_size)
    //     .shuffle(config.seed)
    //     .num_workers(config.num_workers)
    //     .build(BoardDataset::train().unwrap());
    // for data in dataloader_train.iter() {
    //     println!("{}", data.targets);
    // }
}
