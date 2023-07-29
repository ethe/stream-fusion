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
    pub struct Filter<S, P> {
        #[pin]
        pub(crate) stream: S,
        pub(crate) predicate: P,
    }
}

impl<S, P> Stream for Filter<S, P>
where
    S: Stream,
    P: FnMut(&S::Item) -> bool,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let mut this = self.project();
        return Poll::Ready(match ready!(this.stream.as_mut().poll_next(cx)) {
            Step::NotYet => Step::NotYet,
            Step::Ready(item) => {
                if (this.predicate)(&item) {
                    Step::Ready(item)
                } else {
                    Step::NotYet
                }
            }
            Step::Done => Step::Done,
        });
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.stream.size_hint().1)
    }
}
