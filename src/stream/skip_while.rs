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
    pub struct SkipWhile<S, P> {
        #[pin]
        pub(super) stream: S,
        pub(super) predicate: Option<P>,
    }
}

impl<S, P> Stream for SkipWhile<S, P>
where
    S: Stream,
    P: FnMut(&S::Item) -> bool,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(match ready!(this.stream.as_mut().poll_next(cx)) {
            Step::NotYet => Step::NotYet,
            Step::Ready(v) => match this.predicate {
                Some(p) => {
                    if !p(&v) {
                        *this.predicate = None;
                        Step::Ready(v)
                    } else {
                        Step::NotYet
                    }
                }
                None => Step::Ready(v),
            },
            Step::Done => Step::Done,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
