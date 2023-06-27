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
    pub struct StepBy<S> {
        #[pin]
        pub(super) stream: S,
        pub(super) step: usize,
        pub(super) i: usize,
    }
}

impl<S: Stream> Stream for StepBy<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let mut this = self.project();
        Poll::Ready(match ready!(this.stream.as_mut().poll_next(cx)) {
            Step::NotYet => Step::NotYet,
            Step::Ready(v) => {
                if *this.i == 0 {
                    *this.i = *this.step - 1;
                    Step::Ready(v)
                } else {
                    *this.i -= 1;
                    Step::NotYet
                }
            }
            Step::Done => Step::Done,
        })
    }
}
