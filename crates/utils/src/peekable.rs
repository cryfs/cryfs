use std::iter::Peekable;

// TODO Docs, Tests

pub trait PeekableExt {
    fn is_empty(&mut self) -> bool;
}

impl<T: Iterator> PeekableExt for Peekable<T> {
    #[inline]
    fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }
}
