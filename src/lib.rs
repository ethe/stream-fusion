#![feature(impl_trait_in_assoc_type)]
#![feature(iter_next_chunk)]
#![feature(maybe_uninit_uninit_array)]
#![feature(core_intrinsics)]

use std::{future::Future, intrinsics::transmute_unchecked, marker::PhantomData, mem::MaybeUninit};

use futures_lite::{future::yield_now, StreamExt};
use futures_util::{select, stream::FuturesUnordered, FutureExt};
use tokio::task::{JoinError, JoinHandle};

pub enum Step<T> {
    NotYet,
    Ready(T),
    Done,
}

impl<T> Step<T> {
    #[inline]
    pub fn map<G, F>(self, mut f: F) -> Step<G>
    where
        F: FnMut(T) -> G,
    {
        match self {
            Step::NotYet => Step::NotYet,
            Step::Ready(ready) => Step::Ready((f)(ready)),
            Step::Done => Step::Done,
        }
    }

    #[inline]
    pub fn and_then<G, F>(self, mut f: F) -> Step<G>
    where
        F: FnMut(T) -> Step<G>,
    {
        match self {
            Step::NotYet => Step::NotYet,
            Step::Ready(ready) => (f)(ready),
            Step::Done => Step::Done,
        }
    }
}

pub trait Morsel: 'static + Send {
    type Item;

    fn next(&mut self) -> Step<Self::Item>;
}

pub trait Source {
    type Item;
    type Morsel: Morsel<Item = Self::Item>;
    type NextFuture<'next>: 'next + Future<Output = Option<Self::Morsel>>
    where
        Self: 'next;

    fn next(&mut self) -> Self::NextFuture<'_>;
}

pub trait Transformer<I>: 'static + Clone + Send {
    type Item;

    fn next(&mut self, input: I) -> Step<Self::Item>;
}

pub trait Consumer<I>: 'static + Clone + Send {
    type Return: 'static + Send;

    fn consume(&mut self, input: Step<I>) -> Step<()>;

    fn take(self) -> Self::Return;
}

#[derive(Clone)]
pub struct Id;

impl<I> Transformer<I> for Id {
    type Item = I;

    #[inline]
    fn next(&mut self, input: I) -> Step<Self::Item> {
        Step::Ready(input)
    }
}

#[derive(Clone)]
pub struct Map<T, F> {
    upper: T,
    f: F,
}

impl<T, F> Map<T, F> {
    pub(crate) fn new(upper: T, f: F) -> Self {
        Map { upper, f }
    }
}

impl<I, T, G, F> Transformer<I> for Map<T, F>
where
    T: Transformer<I>,
    F: 'static + Fn(T::Item) -> G + Clone + Send,
{
    type Item = G;

    #[inline]
    fn next(&mut self, input: I) -> Step<Self::Item> {
        self.upper.next(input).map(&self.f)
    }
}

#[derive(Clone)]
pub struct Never;

impl<I> Consumer<I> for Never {
    type Return = ();

    #[inline]
    fn consume(&mut self, input: Step<I>) -> Step<()> {
        let _ = input;
        Step::NotYet
    }

    #[inline]
    fn take(self) -> Self::Return {
        ()
    }
}

#[derive(Clone)]
pub struct Reduce<Acc, F> {
    acc: Option<Acc>,
    f: F,
}

impl<Acc, F> Reduce<Acc, F>
where
    F: Fn(Acc, Acc) -> Acc,
{
    pub(crate) fn new(f: F) -> Self {
        Self { acc: None, f }
    }
}

impl<Acc, F> Consumer<Acc> for Reduce<Acc, F>
where
    Acc: 'static + Send + Clone,
    F: 'static + Fn(Acc, Acc) -> Acc + Send + Clone,
{
    type Return = Option<Acc>;

    #[inline]
    fn consume(&mut self, input: Step<Acc>) -> Step<()> {
        match input {
            Step::NotYet => Step::NotYet,
            Step::Ready(item) => {
                match self.acc.take() {
                    Some(acc) => self.acc = Some((self.f)(acc, item)),
                    None => self.acc = Some(item),
                }
                Step::NotYet
            }
            Step::Done => Step::Done,
        }
    }

    #[inline]
    fn take(self) -> Self::Return {
        self.acc
    }
}

pub struct Stream<S, T, C> {
    source: S,
    transformer: T,
    consumer: C,
}

impl<S> Stream<S, Id, Never>
where
    S: Source,
{
    fn new(source: S) -> Self {
        Stream {
            source,
            transformer: Id,
            consumer: Never,
        }
    }
}

impl<S, T, C> Stream<S, T, C>
where
    S: Source,
    T: Transformer<S::Item>,
    C: Consumer<T::Item>,
{
    #[inline]
    pub async fn execute<const N: usize>(&mut self) -> Option<JoinHandle<C::Return>> {
        self.source.next().await.map(|mut morsel| {
            let mut transformer = self.transformer.clone();
            let mut consumer = self.consumer.clone();
            tokio::spawn(async move {
                let mut step = 0;
                loop {
                    if step == N {
                        yield_now().await;
                    }
                    let consumed =
                        consumer.consume(morsel.next().and_then(|item| transformer.next(item)));
                    match consumed {
                        Step::NotYet => {}
                        Step::Done => return consumer.take(),
                        _ => unreachable!(),
                    }
                    step += 1;
                }
            })
        })
    }

    #[inline]
    pub fn map<G, F>(self, f: F) -> Stream<S, Map<T, F>, C>
    where
        F: 'static + Fn(T::Item) -> G + Clone + Send,
    {
        Stream {
            source: self.source,
            transformer: Map::new(self.transformer, f),
            consumer: self.consumer,
        }
    }

    #[inline]
    pub async fn reduce<F, const N: usize>(self, f: F) -> Result<Option<T::Item>, JoinError>
    where
        T::Item: 'static + Send + Clone,
        F: 'static + Fn(T::Item, T::Item) -> T::Item + Send + Clone,
    {
        let mut stream = Stream {
            source: self.source,
            transformer: self.transformer,
            consumer: Reduce::new(f.clone()),
        };
        let mut result = None;

        let mut tasks = FuturesUnordered::new();
        loop {
            select! {
                task = stream.execute::<N>().fuse() => match task {
                    Some(task) => tasks.push(task),
                    None => break,
                },
                task = tasks.next().fuse() => match task {
                    Some(item) => {
                        let item = item?;
                        match result.take() {
                            Some(inner) => {
                                result = item.map(|item| (f)(inner, item));
                            }
                            None => {
                                result = item;
                            }
                        }
                    }
                    _ => {}
                },
            }
        }

        loop {
            match tasks.next().await {
                Some(item) => {
                    let item = item?;
                    match result.take() {
                        Some(inner) => {
                            result = item.map(|item| (f)(inner, item));
                        }
                        None => {
                            result = item;
                        }
                    }
                }
                None => break,
            }
        }

        Ok(result)
    }
}

pub struct ChunkMorsel<T, const N: usize> {
    chunk: [MaybeUninit<T>; N],
    size: usize,
    pos: usize,
}

impl<T: 'static + Send, const N: usize> Morsel for ChunkMorsel<T, N> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Step<Self::Item> {
        if self.pos == self.size {
            return Step::Done;
        }
        let item = unsafe { self.chunk.get_unchecked(self.pos).assume_init_read() };
        self.pos += 1;
        Step::Ready(item)
    }
}

impl<T, const N: usize> ChunkMorsel<T, N> {
    fn new(chunk: [MaybeUninit<T>; N], size: usize) -> Self {
        Self {
            chunk,
            size,
            pos: 0,
        }
    }
}

pub struct WindowIterator<I, const N: usize> {
    iter: I,
    done: bool,
    _marker: PhantomData<[(); N]>,
}

impl<I, const N: usize> WindowIterator<I, N> {
    #[inline]
    pub fn from(iter: I) -> Self {
        WindowIterator {
            iter,
            done: false,
            _marker: PhantomData,
        }
    }
}

impl<I, const N: usize> Source for WindowIterator<I, N>
where
    I: Iterator,
    I::Item: 'static + Send,
{
    type Item = I::Item;

    type Morsel = ChunkMorsel<Self::Item, N>;

    type NextFuture<'next> = impl 'next + Future<Output = Option<Self::Morsel>>
    where
        Self: 'next;

    #[inline]
    fn next(&mut self) -> Self::NextFuture<'_> {
        async {
            if self.done {
                return None;
            }
            Some(match self.iter.next_chunk::<N>() {
                Ok(chunk) => ChunkMorsel::new(unsafe { transmute_unchecked(chunk) }, N),
                Err(iter) => {
                    self.done = true;
                    let mut chunk = MaybeUninit::uninit_array();
                    let mut size = 0;
                    for (id, item) in iter.enumerate() {
                        unsafe {
                            chunk.get_unchecked_mut(id).write(item);
                        }
                        size += 1;
                    }
                    ChunkMorsel::new(chunk, size)
                }
            })
        }
    }
}

pub trait IntoFusion {
    type Source: Source;

    fn fusion(self) -> Stream<Self::Source, Id, Never>;
}

impl<S: Source> IntoFusion for S {
    type Source = Self;

    #[inline]
    fn fusion(self) -> Stream<Self::Source, Id, Never> {
        Stream::new(self)
    }
}

#[cfg(test)]
mod tests {

    use crate::{IntoFusion, WindowIterator};

    #[tokio::test]
    async fn chain() {
        let i = WindowIterator::<_, 512>::from((0..4096).into_iter());
        let result = i
            .fusion()
            .map(|item| item + 1)
            .map(|item| item + 1)
            .reduce::<_, 32>(|r, item| r + item)
            .await
            .unwrap();

        let expect = (0..4096)
            .into_iter()
            .map(|item| item + 1)
            .map(|item| item + 1)
            .reduce(|r, item| r + item);
        assert_eq!(result, expect);
    }
}
