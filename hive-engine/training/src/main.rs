#![recursion_limit = "256"]
mod dataset;
mod model;
mod training;

use crate::model::ModelConfig;
use burn::backend::Wgpu;

fn main() {
    type MyBackend = Wgpu<f32, i32>;

    let device = Default::default();
    let model = ModelConfig::new(1, 256, 32, 1).init::<MyBackend>(&device);

    println!("{model}");
}
