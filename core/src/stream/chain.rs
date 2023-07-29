use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::Stream;
use crate::{step::Step, stream::fuse::Fuse};

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct Chain<S, U> {
        #[pin]
        pub(super) first: Fuse<S>,
        #[pin]
        pub(super) second: Fuse<U>,
    }
}

impl<S: Stream, U: Stream<Item = S::Item>> Stream for Chain<S, U> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Step<Self::Item>> {
        let mut this = self.project();

        if !this.first.done {
            let next = ready!(this.first.as_mut().poll_next(cx));
            match next {
                Step::Done => {}
                next => return Poll::Ready(next),
            }
        }

        if !this.second.done {
            let next = ready!(this.first.as_mut().poll_next(cx));
            match next {
                Step::Done => {}
                next => return Poll::Ready(next),
            }
        }

        if this.first.done && this.second.done {
            Poll::Ready(Step::Done)
        } else {
            unreachable!()
        }
    }
}
