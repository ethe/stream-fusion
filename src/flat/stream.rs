use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    consumer::Reduce,
    stream::{Map, Stream},
};

pub trait FlatStream {
    type Item;
    type Stream: Stream<Item = Self::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Stream>>;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    #[inline]
    fn map<T, F>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Item) -> T + Clone,
        Self: Sized,
    {
        Map::new(self, f)
    }

    #[inline]
    fn reduce<F>(self, f: F) -> Reduce<Self::Item, Self, F>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Clone,
        Self: Sized,
    {
        Reduce::new(self, f)
    }
}

impl<L, T, F> FlatStream for Map<L, F>
where
    L: FlatStream,
    F: Fn(<L::Stream as Stream>::Item) -> T + Clone,
{
    type Item = T;
    type Stream = Map<L::Stream, F>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Stream>> {
        let this = self.project();
        this.upper
            .poll_next(cx)
            .map(|upper| upper.map(|upper| Map::new(upper, this.f.clone())))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.upper.size_hint()
    }
}
