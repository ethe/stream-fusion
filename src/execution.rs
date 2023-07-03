use std::{
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::StreamExt;
use tokio::task::JoinHandle;

use crate::{
    consumer::Consumer,
    flat::{consumer::FlatConsumerExt, FlatConsumer},
    futures,
};

pub trait Execution {
    type Future: Future;

    fn execute(self, n: usize, step_by: usize) -> Self::Future;
}

impl<C> Execution for C
where
    C: FlatConsumer + Unpin,
    C::Consumer: Send + 'static,
    C::Return: Send + 'static,
{
    type Future = impl Future<Output = C::Return>;

    #[inline]
    fn execute(mut self, n: usize, step_by: usize) -> Self::Future {
        async move {
            let mut tasks = Vec::with_capacity(n);
            let mut spawner = self.as_mut();
            for _ in 0..n {
                match spawner.next().await {
                    Some(consumer) => {
                        tasks.push(tokio::spawn(ConsumeFuture::new(consumer, step_by)));
                    }
                    None => break,
                }
            }

            self.merge(TaskStream::new(tasks)).await
        }
    }
}

pub struct ConsumeFuture<C> {
    consumer: MaybeUninit<C>,
    step_by: usize,
    steps: usize,
}

impl<C> ConsumeFuture<C> {
    fn new(consumer: C, step_by: usize) -> Self {
        Self {
            consumer: MaybeUninit::new(consumer),
            step_by,
            steps: 0,
        }
    }
}

impl<C> Future for ConsumeFuture<C>
where
    C: Consumer + Unpin,
{
    type Output = C::Return;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            if self.steps == self.step_by {
                self.steps = 0;
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            match unsafe { Pin::new(self.consumer.assume_init_mut()) }.poll_next(cx) {
                Poll::Ready(result) => match result {
                    Some(_) => {
                        let consumer = unsafe { self.consumer.assume_init_read() };
                        return Poll::Ready(consumer.take());
                    }
                    None => {
                        self.steps += 1;
                        continue;
                    }
                },
                Poll::Pending => {
                    self.steps = 0;
                    return Poll::Pending;
                }
            }
        }
    }
}

pub struct TaskStream<T> {
    tasks: Vec<JoinHandle<T>>,
    already: usize,
}

impl<T> TaskStream<T> {
    fn new(tasks: Vec<JoinHandle<T>>) -> Self {
        Self { tasks, already: 0 }
    }
}

impl<T> futures::Stream for TaskStream<T> {
    type Item = T;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.already == this.tasks.len() {
            return Poll::Ready(None);
        }
        match Pin::new(&mut this.tasks[this.already]).poll(cx) {
            Poll::Ready(ready) => {
                this.already += 1;
                Poll::Ready(Some(ready.unwrap()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
