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

    #[inline]
    pub fn as_ref(&self) -> Step<&T> {
        match self {
            Step::NotYet => Step::NotYet,
            Step::Ready(ready) => Step::Ready(ready),
            Step::Done => Step::Done,
        }
    }

    #[inline]
    pub fn as_mut(&mut self) -> Step<&mut T> {
        match self {
            Step::NotYet => Step::NotYet,
            Step::Ready(ready) => Step::Ready(ready),
            Step::Done => Step::Done,
        }
    }
}

impl<T, E> Step<Result<T, E>> {
    #[inline]
    pub fn transpose(self) -> Result<Step<T>, E> {
        match self {
            Step::NotYet => Ok(Step::NotYet),
            Step::Ready(ready) => match ready {
                Ok(ok) => Ok(Step::Ready(ok)),
                Err(err) => Err(err),
            },
            Step::Done => Ok(Step::Done),
        }
    }
}

impl<T> Step<&T> {
    #[inline]
    pub fn cloned(self) -> Step<T>
    where
        T: Clone,
    {
        match self {
            Step::NotYet => Step::NotYet,
            Step::Ready(v) => Step::Ready(v.clone()),
            Step::Done => Step::Done,
        }
    }

    #[inline]
    pub fn copied(self) -> Step<T>
    where
        T: Copy,
    {
        match self {
            Step::NotYet => Step::NotYet,
            Step::Ready(v) => Step::Ready(v.clone()),
            Step::Done => Step::Done,
        }
    }
}
