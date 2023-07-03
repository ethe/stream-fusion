use std::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{futures, step::Step, stream::Stream};

pub trait Consumer: futures::Stream<Item = ()> {
    type Return;

    fn take(self) -> Self::Return;
}

pin_project! {
    pub struct Reduce<Acc, S, F>
    {
        #[pin]
        pub upper: S,
        pub f: F,
        pub acc: Option<Acc>,
    }
}

impl<Acc, S, F> Reduce<Acc, S, F> {
    pub(crate) fn new(upper: S, f: F) -> Reduce<Acc, S, F> {
        Reduce {
            upper,
            f,
            acc: None,
        }
    }
}

impl<S, F> futures::Stream for Reduce<S::Item, S, F>
where
    S: Stream,
    F: Fn(S::Item, S::Item) -> S::Item,
{
    type Item = ();

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.upper.poll_next(cx) {
            Poll::Ready(item) => Poll::Ready(match item {
                Step::NotYet => None,
                Step::Ready(item) => {
                    match this.acc.take() {
                        Some(acc) => *this.acc = Some((this.f)(acc, item)),
                        None => *this.acc = Some(item),
                    }
                    None
                }
                Step::Done => Some(()),
            }),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S, F> Consumer for Reduce<S::Item, S, F>
where
    S: Stream,
    F: Fn(S::Item, S::Item) -> S::Item,
{
    type Return = Option<S::Item>;

    #[inline]
    fn take(self) -> Self::Return {
        self.acc
    }
}
