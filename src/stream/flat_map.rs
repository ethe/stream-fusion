use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::{map::Map, Stream};
use crate::step::Step;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct FlatMap<S, U, F> {
        #[pin] pub(super) stream: Map<S, F>,
        #[pin] pub(super) inner_stream: Option<U>,
    }
}

impl<S, U, F> Stream for FlatMap<S, U, F>
where
    S: Stream,
    U: Stream,
    F: FnMut(S::Item) -> U,
{
    type Item = U::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let mut this = self.project();

        loop {
            if let Some(inner) = this.inner_stream.as_mut().as_pin_mut() {
                match ready!(inner.poll_next(cx)) {
                    Step::NotYet => return Poll::Ready(Step::NotYet),
                    Step::Ready(item) => return Poll::Ready(Step::Ready(item)),
                    Step::Done => {
                        this.inner_stream.set(None);
                    }
                }
            }

            match ready!(this.stream.as_mut().poll_next(cx)) {
                Step::NotYet => return Poll::Ready(Step::NotYet),
                Step::Ready(stream) => this.inner_stream.set(Some(stream)),
                Step::Done => return Poll::Ready(Step::Done),
            }
        }
    }
}
