use core::{
    cmp::min,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::Stream;
use crate::step::Step;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct Take<S> {
        #[pin]
        pub(super) stream: S,
        pub(super) n: usize,
    }
}

impl<S: Stream> Stream for Take<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<S::Item>> {
        let this = self.project();

        if *this.n == 0 {
            Poll::Ready(Step::Done)
        } else {
            let next = ready!(this.stream.poll_next(cx));
            match next {
                Step::NotYet => return Poll::Ready(Step::NotYet),
                Step::Ready(_) => {
                    *this.n -= 1;
                }
                Step::Done => return Poll::Ready(Step::Done),
            }
            Poll::Ready(next)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.stream.size_hint();
        let max = match size.1 {
            Some(max) => min(max, self.n),
            None => self.n,
        };
        (size.0, Some(max))
    }
}
