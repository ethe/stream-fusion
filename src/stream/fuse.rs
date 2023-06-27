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
    pub struct Fuse<S> {
        #[pin]
        pub(super) stream: S,
        pub(super) done: bool,
    }
}

impl<S: Stream> Stream for Fuse<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<S::Item>> {
        let this = self.project();

        if *this.done {
            Poll::Ready(Step::Done)
        } else {
            let next = ready!(this.stream.poll_next(cx));
            if let Step::Done = next {
                *this.done = true;
            }
            Poll::Ready(next)
        }
    }
}
