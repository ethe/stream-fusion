use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::{ext::StreamExt, Stream};
use crate::step::Step;

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct NextFuture<'a, S: ?Sized> {
    pub(super) stream: &'a mut S,
}

impl<S: Unpin + ?Sized> Unpin for NextFuture<'_, S> {}

impl<S: Stream + Unpin + ?Sized> Future for NextFuture<'_, S> {
    type Output = Step<S::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.stream.poll_next(cx)
    }
}
