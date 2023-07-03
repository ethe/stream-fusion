use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use crate::{flat::FlatStream, step::Step, stream::Stream};

pub struct StreamSync<S> {
    stream: Arc<Mutex<S>>,
}

impl<S> StreamSync<S> {
    #[inline]
    pub fn new(stream: S) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
        }
    }
}

impl<S: Unpin> Unpin for StreamSync<S> {}

impl<S> Clone for StreamSync<S> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            stream: Arc::clone(&self.stream),
        }
    }
}

impl<S: Stream + Unpin> Stream for StreamSync<S> {
    type Item = S::Item;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let mut guard = self.get_mut().stream.lock().unwrap();
        Pin::new(&mut *guard).poll_next(cx)
    }
}

impl<S: Stream + Unpin> FlatStream for StreamSync<S> {
    type Item = S::Item;
    type Stream = Self;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Stream>> {
        Poll::Ready(Some(self.clone()))
    }
}
