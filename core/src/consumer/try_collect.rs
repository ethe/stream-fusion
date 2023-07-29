use core::{
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::Consumer;
use crate::{step::Step, stream::Stream};

pin_project! {
    #[must_use = "comsumers do nothing unless you execute them"]
    pub struct TryCollect<S, C> {
        #[pin]
        pub(crate) stream: S,
        pub(crate) collection: C,
    }
}

impl<T, E, S, C> Consumer for TryCollect<S, C>
where
    S: Stream<Item = Result<T, E>>,
    C: Default + Extend<T>,
{
    type Output = Result<C, (E, C)>;

    fn poll_consume(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Output>> {
        let this = self.project();
        return Poll::Ready(match ready!(this.stream.poll_next(cx)) {
            Step::NotYet => None,
            Step::Ready(e) => match e {
                Ok(e) => {
                    this.collection.extend(Some(e));
                    None
                }
                Err(err) => Some(Err((err, mem::take(this.collection)))),
            },
            Step::Done => Some(Ok(mem::take(this.collection))),
        });
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
