use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::Stream;
use crate::step::Step;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct Cloned<S> {
        #[pin]
        pub(super) stream: S,
    }
}

impl<'a, S, T: 'a> Stream for Cloned<S>
where
    S: Stream<Item = &'a T>,
    T: Clone,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let this = self.project();
        let next = ready!(this.stream.poll_next(cx));
        Poll::Ready(next.cloned())
    }
}
