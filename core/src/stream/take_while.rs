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
    pub struct TakeWhile<S, P> {
        #[pin]
        pub(super) stream: S,
        pub(super) predicate: P,
    }
}

impl<S, P> Stream for TakeWhile<S, P>
where
    S: Stream,
    P: FnMut(&S::Item) -> bool,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let this = self.project();

        Poll::Ready(match ready!(this.stream.poll_next(cx)) {
            Step::NotYet => Step::NotYet,
            Step::Ready(item) => {
                if (this.predicate)(&item) {
                    Step::Ready(item)
                } else {
                    Step::Done
                }
            }
            Step::Done => Step::Done,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
