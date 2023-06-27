use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::ready;
use pin_project_lite::pin_project;

use super::Consumer;
use crate::{step::Step, stream::Stream};

pin_project! {
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Fold<S, F, T> {
        #[pin]
        pub(crate) stream: S,
        pub(crate) f: F,
        pub(crate) acc: Option<T>,
    }
}

impl<S, F, T> Consumer for Fold<S, F, T>
where
    S: Stream,
    F: FnMut(T, S::Item) -> T,
{
    type Output = T;

    fn poll_consume(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Output>> {
        let mut this = self.project();
        Poll::Ready(match ready!(this.stream.as_mut().poll_next(cx)) {
            Step::NotYet => None,
            Step::Ready(v) => {
                let old = this.acc.take().unwrap();
                let new = (this.f)(old, v);
                *this.acc = Some(new);
                None
            }
            Step::Done => Some(this.acc.take().unwrap()),
        })
    }
}
