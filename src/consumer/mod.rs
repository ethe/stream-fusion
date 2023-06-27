pub mod collect;
pub mod count;
pub mod fold;
pub mod partition;
pub mod try_collect;

use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::execution::yield_by::YieldBy;

#[must_use = "comsumers do nothing unless you execute them"]
pub trait Consumer {
    type Output;

    fn poll_consume(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Output>>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub trait ConsumerExt: Consumer {
    fn yield_by(self, step: usize) -> YieldBy<Self>
    where
        Self: Sized,
    {
        assert!(step > 0, "`step` must be greater than zero");

        YieldBy {
            consumer: self,
            n: 0,
            step,
        }
    }
}

impl<C: Consumer + ?Sized> ConsumerExt for C {}
