use std::iter::{DoubleEndedIterator, FusedIterator};

use super::component::PathComponent;
use super::path::AbsolutePath;

pub struct ComponentIter<'a> {
    // This contains the remainder of the path to iterate over.
    // It must uphold the invariants of [AbsolutePath], except
    // - It omits the starting slash if there is one.
    // - It can be empty if there aren't any more components (or if it is just the root path which doesn't have any components to begin with).
    path: &'a str,
}

impl<'a> ComponentIter<'a> {
    #[inline]
    pub(super) fn new(path: &'a AbsolutePath) -> Self {
        let path_str: &str = path.as_ref();
        assert!(path_str.chars().next() == Some('/'));
        Self {
            path: &path_str[1..],
        }
    }
}

impl<'a> Iterator for ComponentIter<'a> {
    type Item = &'a PathComponent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.path.is_empty() {
            return None;
        }

        match self.path.find('/') {
            None => {
                // There is no slash in the path
                let component = PathComponent::new_assert_invariants(self.path);
                self.path = "";
                Some(component)
            }
            Some(index_of_first_separator) => {
                // There is at least one slash in the path
                let component =
                    PathComponent::new_assert_invariants(&self.path[..index_of_first_separator]);
                self.path = &self.path[(index_of_first_separator + 1)..];
                Some(component)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = {
            if self.path.is_empty() {
                0
            } else {
                self.path.matches('/').count() + 1
            }
        };
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    // TODO Add advance_by once it is stable
}

impl DoubleEndedIterator for ComponentIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.path.is_empty() {
            return None;
        }

        match self.path.rfind('/') {
            None => {
                // There is no slash in the path
                let component = PathComponent::new_assert_invariants(self.path);
                self.path = "";
                Some(component)
            }
            Some(index_of_last_separator) => {
                // There is at least one slash in the path
                let component = PathComponent::new_assert_invariants(
                    &self.path[(index_of_last_separator + 1)..],
                );
                self.path = &self.path[..index_of_last_separator];
                Some(component)
            }
        }
    }

    // TODO Add advance_back_by once it is stable
}

impl FusedIterator for ComponentIter<'_> {}

impl ExactSizeIterator for ComponentIter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;
    use rstest_reuse::{self, *};

    #[template]
    #[rstest]
    #[case("/", &[])]
    #[case("/foo", &["foo"])]
    #[case("/foo/bar", &["foo", "bar"])]
    #[case("/foo/bar/baz", &["foo", "bar", "baz"])]
    #[case("/ä/☕/🍕", &["ä", "☕", "🍕"])]
    fn parameters(#[case] path: &str, #[case] expected_components: &[&str]) {}

    #[apply(parameters)]
    fn forward_iteration(path: &str, expected_components: &[&str]) {
        let path = AbsolutePath::new_assert_invariants(path);
        let components: Vec<&str> = path.iter().map(|component| component.as_ref()).collect();
        assert_eq!(expected_components, components.as_slice());
    }

    #[apply(parameters)]
    fn backward_iteration(path: &str, expected_components: &[&str]) {
        let path = AbsolutePath::new_assert_invariants(path);
        let components: Vec<&str> = path
            .iter()
            .rev()
            .map(|component| component.as_ref())
            .collect();
        assert_eq!(
            expected_components
                .iter()
                .rev()
                .copied()
                .collect::<Vec<&str>>()
                .as_slice(),
            components.as_slice()
        );
    }

    #[apply(parameters)]
    fn size_hint(path: &str, expected_components: &[&str]) {
        let path = AbsolutePath::new_assert_invariants(path);
        let (lower, upper) = path.iter().size_hint();
        let expected_len = expected_components.len();
        assert_eq!(expected_len, lower);
        assert_eq!(Some(expected_len), upper);
    }

    #[apply(parameters)]
    fn len(path: &str, expected_components: &[&str]) {
        let path = AbsolutePath::new_assert_invariants(path);
        let expected_len = expected_components.len();
        assert_eq!(expected_len, path.iter().len());
    }

    #[apply(parameters)]
    fn count(path: &str, expected_components: &[&str]) {
        let path = AbsolutePath::new_assert_invariants(path);
        let expected_len = expected_components.len();
        assert_eq!(expected_len, path.iter().count());
    }

    #[apply(parameters)]
    fn next(path: &str, expected_components: &[&str]) {
        let path = AbsolutePath::new_assert_invariants(path);
        let mut iter = path.iter();
        for component in expected_components {
            assert_eq!(*component, iter.next().unwrap().as_str());
        }
        // Make sure it's a FusedIterator
        for _ in 0..10 {
            assert_eq!(None, iter.next());
        }
    }

    #[apply(parameters)]
    fn next_back(path: &str, expected_components: &[&str]) {
        let path = AbsolutePath::new_assert_invariants(path);
        let mut iter = path.iter();
        for component in expected_components.iter().rev() {
            assert_eq!(*component, iter.next_back().unwrap().as_str());
        }
        // Make sure it's a FusedIterator
        for _ in 0..10 {
            assert_eq!(None, iter.next());
        }
    }

    #[apply(parameters)]
    fn last(path: &str, expected_components: &[&str]) {
        let path = AbsolutePath::new_assert_invariants(path);
        assert_eq!(
            expected_components.last().copied(),
            path.iter().last().map(|c| c.as_str())
        );
    }
}
