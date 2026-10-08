#![doc = include_str!("../../../README.md")]

mod console;
mod integrations;
mod model;
mod services;

pub use console::capabilities;
pub use hud_width as width;
pub use hud_width::{cell_width, cell_width_of_cluster, clusters, fold, pad, truncate};
pub use model::{Capabilities, ColorSystem, EnvSnapshot, Stream, StreamInfo};
pub use services::resolve::resolve;
