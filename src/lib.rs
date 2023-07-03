#![feature(impl_trait_in_assoc_type)]

pub mod consumer;
pub mod execution;
pub mod flat;
pub mod step;
pub mod stream;
pub mod sync;

pub mod futures {
    pub use futures_lite::*;
}

#[cfg(test)]
mod tests {
    use futures_lite::stream;

    use crate::{execution::Execution, flat::FlatStream, sync::StreamSync};

    #[tokio::test]
    async fn chain() {
        let sum = StreamSync::new(stream::iter(0..4096))
            .map(|i| i + 1)
            .map(|i| i + 1)
            .reduce(|acc, i| acc + i)
            .execute(8, 256)
            .await;

        println!("{:?}", sum);

        println!(
            "{:?}",
            (0..4096)
                .into_iter()
                .map(|i| i + 1)
                .map(|i| i + 1)
                .sum::<i32>()
        )
    }
}
