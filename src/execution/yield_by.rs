use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::consumer::Consumer;

#[must_use = "futures do nothing unless polled"]
pub struct YieldBy<C: Consumer> {
    pub(crate) consumer: C,
    pub(crate) n: usize,
    pub(crate) step: usize,
}

impl<C> Unpin for YieldBy<C> where C: Consumer + Unpin {}

impl<C: Consumer + Unpin> Future for YieldBy<C> {
    type Output = C::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            if self.n == self.step {
                self.n = 0;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            match Pin::new(&mut self.consumer).poll_consume(cx) {
                Poll::Ready(ret) => {
                    if let Some(ret) = ret {
                        return Poll::Ready(ret);
                    }
                }
                Poll::Pending => {
                    self.n = 0;
                    return Poll::Pending;
                }
            }
            self.n += 1;
        }
    }
}
