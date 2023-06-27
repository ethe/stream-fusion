use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use crate::{step::Step, stream::Stream};

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct Flatten<S: Stream> {
        #[pin]
        pub(super) stream: S,
        #[pin]
        pub(super) inner_stream: Option<S::Item>,
    }
}

impl<S, U> Stream for Flatten<S>
where
    S: Stream<Item = U>,
    U: Stream,
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
                Step::Ready(inner) => this.inner_stream.set(Some(inner)),
                Step::Done => return Poll::Ready(Step::Done),
            }
        }
    }
}
