pub mod chain;
pub mod cloned;
pub mod copied;
pub mod ext;
pub mod filter;
pub mod flat_map;
pub mod flatten;
pub mod fuse;
pub mod map;
pub mod map_async;
pub mod next;
pub mod skip;
pub mod skip_while;
pub mod step_by;
pub mod take;
pub mod take_while;
pub mod try_next;

use core::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::step::Step;

#[must_use = "streams do nothing unless consumed"]
pub trait Stream {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<S: futures_core::Stream> Stream for S {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        S::poll_next(self, cx).map(|item| match item {
            Some(item) => Step::Ready(item),
            None => Step::Done,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        S::size_hint(self)
    }
}

pub trait IntoFusion {
    type Stream: Stream;

    fn into_fusion(self) -> Self::Stream;
}

pin_project! {
    #[derive(Debug)]
    pub struct IteratorStream<I> {
        iterator: I,
    }
}

impl<I: Iterator> From<I> for IteratorStream<I> {
    fn from(value: I) -> Self {
        Self { iterator: value }
    }
}

impl<I: Iterator> Stream for IteratorStream<I> {
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        Poll::Ready(match self.project().iterator.next() {
            Some(item) => Step::Ready(item),
            None => Step::Done,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

impl<I: Iterator> IntoFusion for I {
    type Stream = IteratorStream<I>;

    fn into_fusion(self) -> Self::Stream {
        self.into()
    }
}
