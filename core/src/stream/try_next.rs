use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;

use super::{ext::StreamExt, Stream};
use crate::step::Step;

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct TryNextFuture<'a, S: ?Sized> {
    pub(super) stream: &'a mut S,
}

impl<S: Unpin + ?Sized> Unpin for TryNextFuture<'_, S> {}

impl<T, E, S> Future for TryNextFuture<'_, S>
where
    S: Stream<Item = Result<T, E>> + Unpin + ?Sized,
{
    type Output = Result<Step<T>, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = ready!(self.stream.poll_next(cx));
        Poll::Ready(res.transpose())
    }
}
