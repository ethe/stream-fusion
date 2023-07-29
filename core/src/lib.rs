#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(extend_one))]

pub mod consumer;
pub mod execution;
pub mod step;
pub mod stream;

pub mod prelude {
    pub use crate::{
        consumer::{Consumer, ConsumerExt},
        stream::{ext::StreamExt, IntoFusion, IteratorStream, Stream},
    };
}
