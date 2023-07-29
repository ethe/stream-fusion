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
    pub struct Partition<S, P, B> {
        #[pin]
        pub(crate) stream: S,
        pub(crate) predicate: P,
        pub(crate) res: Option<(B, B)>,
    }
}

impl<S, P, B> Consumer for Partition<S, P, B>
where
    S: Stream + Sized,
    P: FnMut(&S::Item) -> bool,
    B: Default + Extend<S::Item>,
{
    type Output = (B, B);

    fn poll_consume(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Output>> {
        let mut this = self.project();

        Poll::Ready(match ready!(this.stream.as_mut().poll_next(cx)) {
            Step::NotYet => None,
            Step::Ready(v) => {
                let res = this.res.as_mut().unwrap();
                if (this.predicate)(&v) {
                    res.0.extend(Some(v));
                } else {
                    res.1.extend(Some(v));
                }
                None
            }
            Step::Done => Some(this.res.take().unwrap()),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
