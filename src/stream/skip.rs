use core::{
    cmp::max,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::Stream;
use crate::step::Step;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct Skip<S> {
        #[pin]
        pub(super) stream: S,
        pub(super) n: usize,
    }
}

impl<S: Stream> Stream for Skip<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(match ready!(this.stream.as_mut().poll_next(cx)) {
            Step::NotYet => Step::NotYet,
            Step::Ready(item) => match *this.n {
                0 => return Poll::Ready(Step::Ready(item)),
                _ => {
                    *this.n -= 1;
                    Step::NotYet
                }
            },
            Step::Done => Step::Done,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.stream.size_hint();
        (
            max(0, size.0 - self.n),
            size.1.map(|size| max(0, size - self.n)),
        )
    }
}
