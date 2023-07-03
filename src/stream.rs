use std::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{futures, step::Step};

pub trait Stream {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl<S: futures::Stream> Stream for S {
    type Item = S::Item;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        S::poll_next(self, cx).map(|item| match item {
            Some(item) => Step::Ready(item),
            None => Step::Done,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        S::size_hint(self)
    }
}

pin_project! {
    pub struct Map<S, F> {
        #[pin]
        pub upper: S,
        pub f: F,
    }
}

impl<S, F> Map<S, F> {
    pub(crate) fn new(upper: S, f: F) -> Map<S, F> {
        Map { upper, f }
    }
}

impl<S, T, F> Stream for Map<S, F>
where
    S: Stream,
    F: Fn(S::Item) -> T,
{
    type Item = T;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let this = self.project();
        this.upper.poll_next(cx).map(|item| item.map(&*this.f))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.upper.size_hint()
    }
}
