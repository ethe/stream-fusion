use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::Consumer;
use crate::{step::Step, stream::Stream};

pin_project! {
    #[must_use = "comsumers do nothing unless you execute them"]
    pub struct Count<S: ?Sized> {
        pub(crate) count: usize,
        #[pin]
        pub(crate) stream: S,
    }
}

impl<S: Stream + ?Sized> Consumer for Count<S> {
    type Output = usize;

    fn poll_consume(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Output>> {
        let this = self.project();
        Poll::Ready(match ready!(this.stream.poll_next(cx)) {
            Step::NotYet => None,
            Step::Ready(_) => {
                *this.count += 1;
                None
            }
            Step::Done => Some(*this.count),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
