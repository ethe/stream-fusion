use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::Stream;
use crate::step::Step;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct MapAsync<S, F, Fut> {
        #[pin]
        pub(super) stream: S,
        #[pin]
        pub(super) future: Option<Fut>,
        pub(super) f: F,
    }
}

impl<S, F, Fut> Stream for MapAsync<S, F, Fut>
where
    S: Stream,
    F: FnMut(S::Item) -> Fut,
    Fut: Future,
{
    type Item = Fut::Output;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let mut this = self.project();

        loop {
            if let Some(fut) = this.future.as_mut().as_pin_mut() {
                let item = ready!(fut.poll(cx));
                this.future.set(None);
                return Poll::Ready(Step::Ready(item));
            } else {
                match ready!(this.stream.as_mut().poll_next(cx)) {
                    Step::NotYet => return Poll::Ready(Step::NotYet),
                    Step::Ready(item) => this.future.set(Some((this.f)(item))),
                    Step::Done => return Poll::Ready(Step::NotYet),
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let future_len = self.future.is_some() as usize;
        let (lower, upper) = self.stream.size_hint();
        let lower = lower.saturating_add(future_len);
        let upper = upper.and_then(|u| u.checked_add(future_len));
        (lower, upper)
    }
}
