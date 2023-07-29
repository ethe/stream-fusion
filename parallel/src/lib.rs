use fusion_core::stream::{IteratorStream, Stream};

pub trait IndexedParallelStream {
    type Stream: Stream;

    type Streams: Iterator<Item = Self::Stream>;

    fn fusion_split(self, n: usize) -> Self::Streams;
}

pub struct Split<'slice, T> {
    slice: &'slice [T],
    step: usize,
    n: usize,
}

impl<'slice, T> Iterator for Split<'slice, T> {
    type Item = IteratorStream<core::slice::Iter<'slice, T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n >= self.slice.len() {
            return None;
        }
        let mut end = self.n + self.step;
        if self.slice.len() - end <= self.step {
            end = self.slice.len();
        }
        let s = &self.slice[self.n..end];
        self.n = end;
        Some(s.into_iter().into())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut size = self.slice.len() / self.step;
        if self.slice.len() % self.step > 0 {
            size += 1;
        }
        (size, Some(size))
    }
}

impl<'slice, T> ExactSizeIterator for Split<'slice, T> {}

impl<'slice, T> IndexedParallelStream for &'slice [T] {
    type Stream = IteratorStream<core::slice::Iter<'slice, T>>;

    type Streams = Split<'slice, T>;

    fn fusion_split(self, n: usize) -> Self::Streams {
        Split {
            slice: self,
            step: self.len() / n,
            n: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IndexedParallelStream;

    #[test]
    fn test() {
        let a = vec![1, 2, 3];
        let mut i = a.fusion_split(2);
        loop {
            let item = i.next();
            match item {
                Some(item) => {
                    dbg!(item);
                }
                None => break,
            }
        }
    }
}
