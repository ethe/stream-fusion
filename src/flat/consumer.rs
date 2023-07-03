use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use super::FlatStream;
use crate::{
    consumer::{Consumer, Reduce},
    futures,
};

pub trait FlatConsumer {
    type Return;

    type Consumer: Consumer<Return = Self::Return> + Unpin;

    type MergeFuture<S>: Future<Output = Self::Return>
    where
        S: futures::Stream<Item = Self::Return>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Consumer>>;

    fn merge<S>(self, stream: S) -> Self::MergeFuture<S>
    where
        S: futures::Stream<Item = Self::Return>;
}

impl<L, F> FlatConsumer for Reduce<L::Item, L, F>
where
    L: FlatStream,
    L::Stream: Unpin,
    F: Fn(L::Item, L::Item) -> L::Item + Clone,
{
    type Return = Option<L::Item>;

    type Consumer = Reduce<L::Item, L::Stream, F>;

    type MergeFuture<S> = ReduceMerge<S, F>
    where
        S: futures::Stream<Item = Self::Return>;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Consumer>> {
        let this = self.project();
        this.upper
            .poll_next(cx)
            .map(|stream| stream.map(|stream| Reduce::new(stream, this.f.clone())))
    }

    #[inline]
    fn merge<S>(self, stream: S) -> Self::MergeFuture<S>
    where
        S: futures::Stream<Item = Self::Return>,
    {
        ReduceMerge { stream, f: self.f }
    }
}

pin_project! {
    pub struct ReduceMerge<S, F> {
        #[pin]
        stream: S,
        f: F,
    }
}

impl<T, S, F> Future for ReduceMerge<S, F>
where
    F: Fn(T, T) -> T,
    S: futures::Stream<Item = Option<T>>,
{
    type Output = Option<T>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        let mut merge = None;
        loop {
            match this.stream.as_mut().poll_next(cx) {
                Poll::Ready(item) => match item {
                    Some(item) => {
                        merge = match merge.take() {
                            Some(merge) => item.map(|item| (this.f)(merge, item)),
                            None => item,
                        };
                    }
                    None => return Poll::Ready(merge),
                },
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

pub trait FlatConsumerExt: FlatConsumer {
    #[inline]
    fn as_mut(&mut self) -> ConsumeStream<Self>
    where
        Self: Sized,
    {
        ConsumeStream { execution: self }
    }
}

impl<E: FlatConsumer + Unpin + ?Sized> FlatConsumerExt for E {}

pub struct ConsumeStream<'e, E: ?Sized> {
    execution: &'e mut E,
}

impl<'e, E> futures::Stream for ConsumeStream<'e, E>
where
    E: FlatConsumer + Unpin + ?Sized,
{
    type Item = E::Consumer;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut *self.execution).poll_next(cx)
    }
}
