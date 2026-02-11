#![recursion_limit = "256"]
mod model;

use crate::model::ModelConfig;
use burn::backend::{Vulkan, Wgpu};

fn main() {
    type MyBackend = Vulkan<f32, i32>;

    let device = Default::default();
    let model = ModelConfig::new(1, 3073).init::<MyBackend>(&device);

    println!("{model}");
}