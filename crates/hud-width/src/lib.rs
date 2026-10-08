#![no_std]
#![doc = include_str!("../README.md")]

mod fold;
mod props;
mod segment;
mod tables;
mod width;

pub use fold::{Align, Fold, Pad, fold, pad, truncate};
pub use segment::{Clusters, clusters};
pub use width::{cell_width, cell_width_of_cluster, char_width};

/// The Unicode version of the data the tables were generated from, as `(major, minor, patch)`.
pub const UNICODE_VERSION: (u8, u8, u8) = tables::UNICODE_VERSION;
