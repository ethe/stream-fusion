use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::{
    chain::Chain, cloned::Cloned, filter::Filter, flat_map::FlatMap, flatten::Flatten, fuse::Fuse,
    map::Map, map_async::MapAsync, next::NextFuture, skip::Skip, skip_while::SkipWhile,
    step_by::StepBy, take::Take, take_while::TakeWhile, try_next::TryNextFuture, Stream,
};
use crate::{
    consumer::{
        collect::Collect, count::Count, fold::Fold, partition::Partition, try_collect::TryCollect,
    },
    step::Step,
};

/// Extension trait for [`Stream`].
pub trait StreamExt: Stream {
    /// A convenience for calling [`Stream::poll_next()`] on `!`[`Unpin`] types.
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Step<Self::Item>>
    where
        Self: Unpin,
    {
        Stream::poll_next(Pin::new(self), cx)
    }

    /// Retrieves the next item in the stream.
    ///
    /// Returns [`None`] when iteration is finished. Stream implementations may choose to or not to
    /// resume iteration after that.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures_lite::stream::{self, StreamExt};
    ///
    /// # spin_on::spin_on(async {
    /// let mut s = stream::iter(1..=3);
    ///
    /// assert_eq!(s.next().await, Some(1));
    /// assert_eq!(s.next().await, Some(2));
    /// assert_eq!(s.next().await, Some(3));
    /// assert_eq!(s.next().await, None);
    /// # });
    /// ```
    fn next(&mut self) -> NextFuture<'_, Self>
    where
        Self: Unpin,
    {
        NextFuture { stream: self }
    }

    /// Retrieves the next item in the stream.
    ///
    /// This is similar to the [`next()`][`StreamExt::next()`] method, but returns
    /// `Result<Option<T>, E>` rather than `Option<Result<T, E>>`.
    ///
    /// Note that `s.try_next().await` is equivalent to `s.next().await.transpose()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures_lite::stream::{self, StreamExt};
    ///
    /// # spin_on::spin_on(async {
    /// let mut s = stream::iter(vec![Ok(1), Ok(2), Err("error")]);
    ///
    /// assert_eq!(s.try_next().await, Ok(Some(1)));
    /// assert_eq!(s.try_next().await, Ok(Some(2)));
    /// assert_eq!(s.try_next().await, Err("error"));
    /// assert_eq!(s.try_next().await, Ok(None));
    /// # });
    /// ```
    fn try_next<T, E>(&mut self) -> TryNextFuture<'_, Self>
    where
        Self: Stream<Item = Result<T, E>> + Unpin,
    {
        TryNextFuture { stream: self }
    }

    fn count(self) -> Count<Self>
    where
        Self: Sized,
    {
        Count {
            stream: self,
            count: 0,
        }
    }

    fn map<T, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> T,
    {
        Map { stream: self, f }
    }

    fn flat_map<U, F>(self, f: F) -> FlatMap<Self, U, F>
    where
        Self: Sized,
        U: Stream,
        F: FnMut(Self::Item) -> U,
    {
        FlatMap {
            stream: self.map(f),
            inner_stream: None,
        }
    }

    fn flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Item: Stream,
    {
        Flatten {
            stream: self,
            inner_stream: None,
        }
    }

    fn map_async<F, Fut>(self, f: F) -> MapAsync<Self, F, Fut>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Fut,
        Fut: Future,
    {
        MapAsync {
            stream: self,
            future: None,
            f,
        }
    }

    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        Filter {
            stream: self,
            predicate,
        }
    }

    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take { stream: self, n }
    }

    fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        TakeWhile {
            stream: self,
            predicate,
        }
    }

    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        Skip { stream: self, n }
    }

    fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        SkipWhile {
            stream: self,
            predicate: Some(predicate),
        }
    }

    fn step_by(self, step: usize) -> StepBy<Self>
    where
        Self: Sized,
    {
        assert!(step > 0, "`step` must be greater than zero");
        StepBy {
            stream: self,
            step,
            i: 0,
        }
    }

    fn fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        Fuse {
            stream: self,
            done: false,
        }
    }

    fn chain<U>(self, other: U) -> Chain<Self, U>
    where
        Self: Sized,
        U: Stream<Item = Self::Item> + Sized,
    {
        Chain {
            first: self.fuse(),
            second: other.fuse(),
        }
    }

    fn cloned<'a, T>(self) -> Cloned<Self>
    where
        Self: Stream<Item = &'a T> + Sized,
        T: Clone + 'a,
    {
        Cloned { stream: self }
    }

    fn collect<C>(self) -> Collect<Self, C>
    where
        Self: Sized,
        C: Default + Extend<Self::Item>,
    {
        #[allow(unused_mut)]
        let mut collection: C = Default::default();
        #[cfg(feature = "nightly")]
        collection.extend_reserve(self.size_hint().0);
        Collect {
            stream: self,
            collection,
        }
    }

    fn try_collect<T, E, C>(self) -> TryCollect<Self, C>
    where
        Self: Stream<Item = Result<T, E>> + Sized,
        C: Default + Extend<T>,
    {
        #[allow(unused_mut)]
        let mut collection: C = Default::default();
        #[cfg(feature = "nightly")]
        collection.extend_reserve(self.size_hint().0);
        TryCollect {
            stream: self,
            collection,
        }
    }

    fn partition<B, P>(self, predicate: P) -> Partition<Self, P, B>
    where
        Self: Sized,
        B: Default + Extend<Self::Item>,
        P: FnMut(&Self::Item) -> bool,
    {
        Partition {
            stream: self,
            predicate,
            res: Some(Default::default()),
        }
    }

    fn fold<T, F>(self, init: T, f: F) -> Fold<Self, F, T>
    where
        Self: Sized,
        F: FnMut(T, Self::Item) -> T,
    {
        Fold {
            stream: self,
            f,
            acc: Some(init),
        }
    }
}

impl<S: Stream + ?Sized> StreamExt for S {}
