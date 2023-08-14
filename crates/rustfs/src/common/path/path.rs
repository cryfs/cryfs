use derive_more::Display;
use std::borrow::{Borrow, ToOwned};
use std::iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator};
use std::ops::Deref;
use std::str::FromStr;

use super::component::PathComponent;
use super::error::ParsePathError;
use super::iter::ComponentIter;

/// An [AbsolutePath] is similar to [std::path::Path] but adds a few invariants, e.g. that the path must be absolute.
///
/// Similar to [std::path::Path], this type is usually passed around by reference. The owned version of this type is [AbsolutePathBuf].
///
/// # Invariants:
///   - Must be value UTF-8
///   - Must start with '/' (this also means it cannot be empty)
///   - Must not contain any '\' or '\0' characters
///   - Must not contain any empty components (i.e. two slashes following each other)
///   - Must not contain any '.' or '..' components
///   - Must not contain any trailing slashes, except if it is the root path "/" itself
#[derive(PartialEq, Eq, Hash, Debug, Display)]
#[repr(transparent)]
pub struct AbsolutePath {
    path: str,
}

impl AbsolutePath {
    #[inline]
    fn new_without_invariant_check(path: &str) -> &Self {
        unsafe { &*(path as *const str as *const AbsolutePath) }
    }

    #[inline]
    pub(super) fn new_assert_invariants(name: &str) -> &Self {
        Self::check_invariants(name).unwrap();
        Self::new_without_invariant_check(name)
    }

    #[inline]
    pub fn try_from_str(path: &str) -> Result<&Self, ParsePathError> {
        AbsolutePath::check_invariants(path)?;
        Ok(AbsolutePath::new_without_invariant_check(path))
    }

    #[inline]
    pub fn root() -> &'static Self {
        Self::new_assert_invariants("/")
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        &self.path == "/"
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.path
    }

    // TODO Test is_ancestor_of, including the case that "/fo" is not an ancestor of "/foo"
    #[inline]
    pub fn is_ancestor_of(&self, other: &AbsolutePath) -> bool {
        match other.path.strip_prefix(&self.path) {
            Some(remaining) => remaining.starts_with('/'),
            None => false,
        }
    }

    // Tests for [Self::iter] are in the [super::iter] module
    #[inline]
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &PathComponent> + DoubleEndedIterator + FusedIterator + ExactSizeIterator
    {
        self.into_iter()
    }

    fn check_invariants(path: &str) -> Result<(), ParsePathError> {
        if path == "/" {
            return Ok(());
        }

        let mut chars = path.char_indices();
        if chars.next() != Some((0, '/')) {
            if path.is_empty() {
                return Err(ParsePathError::EmptyComponent);
            } else if path.contains(|c| c == '\\' || c == '\0') {
                // Even if it's not absolute, InvalidFormat takes precedence.
                return Err(ParsePathError::InvalidFormat);
            } else {
                return Err(ParsePathError::NotAbsolute);
            }
        }
        let mut current_component_start = 1;
        for (index, character) in chars {
            match character {
                '\\' | '\0' => return Err(ParsePathError::InvalidFormat),
                '/' => {
                    PathComponent::check_invariants_except_contains_slash_or_null(
                        &path[current_component_start..index],
                    )?;
                    current_component_start = index + 1;
                }
                _ => {}
            }
        }
        PathComponent::check_invariants_except_contains_slash_or_null(
            &path[current_component_start..],
        )?;
        Ok(())
    }

    /// Split the path into its parent directory and the last component (e.g. file name).
    ///
    /// Returns `None` if it is called on the root path "/" because that path doesn't have a parent path.
    ///
    /// # Examples
    /// ```
    /// use cryfs_rustfs::AbsolutePath;
    ///
    /// let path = AbsolutePath::try_from_str("/foo/bar").unwrap();
    ///
    /// let (parent, child) = path.split_last().unwrap();
    /// assert_eq!(parent.as_str(), "/foo");
    /// assert_eq!(child.as_str(), "bar");
    ///
    /// let path = AbsolutePath::try_from_str("/").unwrap();
    /// assert_eq!(None, path.split_last());
    /// ```
    pub fn split_last(&self) -> Option<(&AbsolutePath, &PathComponent)> {
        if self.is_root() {
            return None;
        }

        let index_of_last_separator = self
            .path
            .rfind('/')
            .expect("Absolute paths must always have at least one slash");

        if index_of_last_separator == 0 {
            let parent = AbsolutePath::root();
            let child = PathComponent::new_assert_invariants(&self.path[1..]);
            return Some((parent, child));
        }

        let (parent, child) = self.path.split_at(index_of_last_separator);
        let parent = AbsolutePath::new_assert_invariants(parent);
        let child = PathComponent::new_assert_invariants(&child[1..]);
        Some((parent, child))
    }
}

impl<'a> TryFrom<&'a str> for &'a AbsolutePath {
    type Error = ParsePathError;

    #[inline]
    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        AbsolutePath::try_from_str(path)
    }
}

impl<'a> TryFrom<&'a std::path::Path> for &'a AbsolutePath {
    type Error = ParsePathError;

    #[inline]
    fn try_from(path: &'a std::path::Path) -> Result<Self, Self::Error> {
        let path = path.to_str().ok_or(ParsePathError::NotUtf8)?;
        AbsolutePath::try_from_str(path)
    }
}

impl<'a> From<&'a AbsolutePath> for &'a std::path::Path {
    #[inline]
    fn from(path: &'a AbsolutePath) -> Self {
        path.as_ref()
    }
}

impl<'a> From<&'a AbsolutePath> for &'a str {
    #[inline]
    fn from(path: &'a AbsolutePath) -> Self {
        path.as_ref()
    }
}

impl Deref for AbsolutePath {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl AsRef<str> for AbsolutePath {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.path
    }
}

impl AsRef<std::path::Path> for AbsolutePath {
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        self.path.as_ref()
    }
}

impl ToOwned for AbsolutePath {
    type Owned = AbsolutePathBuf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        AbsolutePathBuf {
            path: self.path.to_owned(),
        }
    }
}

// TODO IntoIterator for AbsolutePathBuf could take ownership
impl<'a> IntoIterator for &'a AbsolutePath {
    type Item = &'a PathComponent;
    type IntoIter = ComponentIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ComponentIter::new(self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
pub struct AbsolutePathBuf {
    path: String,
}

impl AbsolutePathBuf {
    #[inline]
    fn new_without_invariant_check(path: String) -> Self {
        Self { path }
    }

    #[inline]
    pub fn root() -> Self {
        AbsolutePath::root().to_owned()
    }

    #[inline]
    pub fn try_from_string(path: String) -> Result<Self, ParsePathError> {
        AbsolutePath::check_invariants(&path)?;
        Ok(AbsolutePathBuf::new_without_invariant_check(path))
    }

    #[inline]
    pub fn push(mut self, component: &PathComponent) -> Self {
        if self.path != "/" {
            self.path.push('/');
        }
        self.path.push_str(component);
        self
    }

    // TODO Test
    #[inline]
    pub fn push_all(mut self, components: &AbsolutePath) -> Self {
        if self.is_root() {
            self.path = components.path.to_owned();
        } else {
            self.path.push_str(&components.path);
        }
        self
    }
}

impl Borrow<AbsolutePath> for AbsolutePathBuf {
    #[inline]
    fn borrow(&self) -> &AbsolutePath {
        AbsolutePath::new_without_invariant_check(&self.path)
    }
}

impl TryFrom<String> for AbsolutePathBuf {
    type Error = ParsePathError;

    #[inline]
    fn try_from(path: String) -> Result<Self, Self::Error> {
        AbsolutePath::check_invariants(&path)?;
        Ok(AbsolutePathBuf::new_without_invariant_check(path))
    }
}

impl TryFrom<std::path::PathBuf> for AbsolutePathBuf {
    type Error = ParsePathError;

    #[inline]
    fn try_from(path: std::path::PathBuf) -> Result<Self, Self::Error> {
        let path = path
            .into_os_string()
            .into_string()
            .map_err(|_| ParsePathError::NotUtf8)?;
        TryFrom::<String>::try_from(path)
    }
}

impl From<AbsolutePathBuf> for std::path::PathBuf {
    #[inline]
    fn from(path: AbsolutePathBuf) -> Self {
        path.path.into()
    }
}

impl From<AbsolutePathBuf> for String {
    #[inline]
    fn from(path: AbsolutePathBuf) -> Self {
        path.path
    }
}

impl FromStr for AbsolutePathBuf {
    type Err = ParsePathError;

    #[inline]
    fn from_str(path: &str) -> Result<Self, Self::Err> {
        Ok(AbsolutePath::try_from_str(path)?.to_owned())
    }
}

impl AsRef<std::path::Path> for AbsolutePathBuf {
    #[inline]
    fn as_ref(&self) -> &std::path::Path {
        self.path.as_ref()
    }
}

impl Deref for AbsolutePathBuf {
    type Target = AbsolutePath;

    #[inline]
    fn deref(&self) -> &Self::Target {
        AbsolutePath::new_without_invariant_check(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root() {
        let path = AbsolutePath::root();
        assert_eq!("/", path.as_str());

        let path = AbsolutePathBuf::root();
        assert_eq!("/", path.as_str());
    }

    #[test]
    fn is_root() {
        assert!(AbsolutePath::root().is_root());
        assert!(AbsolutePath::try_from_str("/").unwrap().is_root());
        assert!(!AbsolutePath::try_from_str("/a").unwrap().is_root());
        assert!(!AbsolutePath::try_from_str("/a/b").unwrap().is_root());
        assert!(!AbsolutePath::try_from_str("/a/b/c").unwrap().is_root());
    }

    #[test]
    fn push() {
        let path = AbsolutePathBuf::root().push("a".try_into().unwrap());
        assert_eq!("/a", path.as_str());

        let path = AbsolutePathBuf::root()
            .push("foo".try_into().unwrap())
            .push("bar".try_into().unwrap());
        assert_eq!("/foo/bar", path.as_str());

        let path = AbsolutePathBuf::root()
            .push("foo".try_into().unwrap())
            .push("bar".try_into().unwrap())
            .push("baz".try_into().unwrap());
        assert_eq!("/foo/bar/baz", path.as_str());
    }

    #[allow(non_snake_case)]
    mod try_from {
        use super::*;

        fn _test(expected: Result<&AbsolutePath, ParsePathError>, path: &str) {
            // AbsolutePath::try_from_str
            let result = AbsolutePath::try_from_str(path);
            assert_eq!(expected, result);

            // TryFrom<&str> for AbsolutePath
            let result = <&AbsolutePath>::try_from(path);
            assert_eq!(expected, result);

            // TryFrom<&std::path::Path> for AbsolutePath
            let result = <&AbsolutePath>::try_from(std::path::Path::new(path));
            assert_eq!(expected, result);

            // AsRef<str> for AbsolutePath
            if expected.is_ok() {
                let result = AbsolutePath::try_from_str(path).unwrap();
                let result: &str = result.as_ref();
                assert_eq!(path, result);
            }

            // AsRef<Path> for AbsolutePath
            if expected.is_ok() {
                let result = AbsolutePath::try_from_str(path).unwrap();
                let result: &std::path::Path = result.as_ref();
                assert_eq!(std::path::Path::new(path), result);
            }

            // From<&AbsolutePath> for &Path
            if expected.is_ok() {
                let result = AbsolutePath::try_from_str(path).unwrap();
                let result: &std::path::Path = <&std::path::Path>::from(result);
                assert_eq!(std::path::Path::new(path), result);
            }

            // From<&AbsolutePath> for &str
            if expected.is_ok() {
                let result = AbsolutePath::try_from_str(path).unwrap();
                let result: &str = <&str>::from(result);
                assert_eq!(path, result);
            }

            // AbsolutePath::as_str
            if expected.is_ok() {
                let result = AbsolutePath::try_from_str(path).unwrap();
                assert_eq!(path, result.as_str());
            }

            // Borrow for AbsolutePathBuf
            if let Ok(expected) = expected {
                let result = AbsolutePathBuf::try_from_string(path.to_string()).unwrap();
                let result = result.borrow();
                assert_eq!(expected, result);
            }

            // Deref for AbsolutePathBuf
            if let Ok(expected) = expected {
                let result = AbsolutePathBuf::try_from_string(path.to_string()).unwrap();
                let result: &AbsolutePath = result.deref();
                assert_eq!(expected, result);
            }

            let expected = expected.map(|s| s.to_owned());

            // AbsolutePathBuf::try_from_string
            let result = AbsolutePathBuf::try_from_string(path.to_string());
            assert_eq!(expected, result);

            // TryFrom<String> for AbsolutePathBuf
            let result = AbsolutePathBuf::try_from(path.to_string());
            assert_eq!(expected, result);

            // TryFrom<std::path::PathBuf> for AbsolutePathBuf
            let result = AbsolutePathBuf::try_from(std::path::PathBuf::from_str(path).unwrap());
            assert_eq!(expected, result);

            // FromStr for AbsolutePathBuf
            let result = AbsolutePathBuf::from_str(path);
            assert_eq!(expected, result);

            // From<AbsolutePathBuf> for String
            if expected.is_ok() {
                let result = AbsolutePathBuf::from_str(path).unwrap();
                assert_eq!(path, String::from(result));
            }

            // From<AbsolutePathBuf> for PathBuf
            if expected.is_ok() {
                let result = AbsolutePathBuf::from_str(path).unwrap();
                assert_eq!(
                    std::path::PathBuf::from(path.to_owned()),
                    std::path::PathBuf::from(result),
                );
            }

            // AbsolutePath::new_without_invariant_check
            let result = AbsolutePath::new_without_invariant_check(path);
            assert_eq!(path, result.as_str());

            // AbsolutePath::new_assert_invariants
            match &expected {
                Ok(expected) => {
                    let result = AbsolutePath::new_assert_invariants(path);
                    assert_eq!(&**expected, result);
                }
                Err(_) => {
                    let result = std::panic::catch_unwind(|| {
                        AbsolutePath::new_assert_invariants(path);
                    });
                    assert!(result.is_err());
                }
            }

            // AbsolutePathBuf::new_without_invariant_check
            let result = AbsolutePathBuf::new_without_invariant_check(path.to_string());
            assert_eq!(path, result.as_str());

            // ToOwned for AbsolutePath
            if let Ok(expected) = &expected {
                let result = AbsolutePath::try_from_str(path).unwrap();
                let result: AbsolutePathBuf = result.to_owned();
                assert_eq!(expected, &result,);
            }

            // AsRef<Path> for AbsolutePathBuf
            if expected.is_ok() {
                let result = AbsolutePathBuf::try_from_string(path.to_owned()).unwrap();
                let result: &std::path::Path = result.as_ref();
                assert_eq!(std::path::Path::new(path), result);
            }

            // Deref for AbsolutePath
            if expected.is_ok() {
                let result = AbsolutePath::try_from_str(path).unwrap();
                let result: &str = result.deref();
                assert_eq!(path, result);
            }
        }

        fn test(expected: Result<&AbsolutePath, ParsePathError>, path: &str) {
            _test(expected, path);
            if path.contains("/") {
                _test(Err(ParsePathError::InvalidFormat), &path.replace("/", "\\"));
            }
        }

        fn _test_non_utf8(path: &[u8]) {
            use std::os::unix::ffi::OsStrExt;
            let path = std::path::Path::new(std::ffi::OsStr::from_bytes(path));

            // TryFrom<&std::path::Path> for AbsolutePath
            let result = <&AbsolutePath>::try_from(path);
            assert_eq!(Err(ParsePathError::NotUtf8), result);

            // TryFrom<std::path::PathBuf> for AbsolutePathBuf
            let result = AbsolutePathBuf::try_from(path.to_owned());
            assert_eq!(Err(ParsePathError::NotUtf8), result);
        }

        fn test_non_utf8(path: &[u8]) {
            _test_non_utf8(path);
            if path.contains(&b'/') {
                _test_non_utf8(
                    &path
                        .into_iter()
                        .map(|a| if a == &b'/' { b'\\' } else { *a })
                        .collect::<Vec<u8>>(),
                );
            }
        }

        #[test]
        fn empty_path() {
            test(Err(ParsePathError::EmptyComponent), "");
        }

        #[test]
        fn double_slash() {
            test(Err(ParsePathError::EmptyComponent), "//");
            test(Err(ParsePathError::EmptyComponent), "//folder");
            test(Err(ParsePathError::EmptyComponent), "//folder/");
            test(Err(ParsePathError::EmptyComponent), "/folder//something");
            test(Err(ParsePathError::EmptyComponent), "/folder//something/");
            test(Err(ParsePathError::EmptyComponent), "/folder//");
        }

        #[test]
        fn disk() {
            test(Err(ParsePathError::NotAbsolute), "C:");
            test(Err(ParsePathError::NotAbsolute), "C:/");
            test(Err(ParsePathError::NotAbsolute), "C://");
            test(Err(ParsePathError::NotAbsolute), "C:/foo");
            test(Err(ParsePathError::NotAbsolute), "C:/foo/");
        }

        #[test]
        fn invalid() {
            test(Err(ParsePathError::NotAbsolute), ":");
            test(Err(ParsePathError::NotAbsolute), ":/foo");
        }

        #[test]
        fn cur_dir() {
            test(Err(ParsePathError::NotAbsolute), ".");
            test(Err(ParsePathError::NotAbsolute), "./");
            test(Err(ParsePathError::NotAbsolute), "./foo");
            test(Err(ParsePathError::NotAbsolute), "./foo/");
            test(Err(ParsePathError::NotAbsolute), "/foo/.");
            test(Err(ParsePathError::NotAbsolute), "/foo/./");
        }

        #[test]
        fn parent_dir() {
            test(Err(ParsePathError::NotAbsolute), "..");
            test(Err(ParsePathError::NotAbsolute), "../");
            test(Err(ParsePathError::NotAbsolute), "../foo");
            test(Err(ParsePathError::NotAbsolute), "../foo/");
            test(Err(ParsePathError::NotAbsolute), "/foo/..");
            test(Err(ParsePathError::NotAbsolute), "/foo/../");
        }

        #[test]
        fn trailing_slash() {
            test(Err(ParsePathError::EmptyComponent), "/first/");
            test(Err(ParsePathError::EmptyComponent), "/first/second/");
        }

        #[test]
        fn success() {
            test(Ok(AbsolutePath::root()), "/");
            test(
                Ok(&AbsolutePathBuf::root().push(<&PathComponent>::try_from("first").unwrap())),
                "/first",
            );
            test(
                Ok(&AbsolutePathBuf::root()
                    .push(<&PathComponent>::try_from("first").unwrap())
                    .push(<&PathComponent>::try_from("second").unwrap())),
                "/first/second",
            );
        }

        #[test]
        fn allowed_special_characters() {
            fn test_special_component(component: &str) {
                test(
                    Ok(&AbsolutePathBuf::root()
                        .push(<&PathComponent>::try_from(component).unwrap())),
                    &format!("/{component}"),
                );
                test(
                    Ok(&AbsolutePathBuf::root()
                        .push(<&PathComponent>::try_from(component).unwrap())
                        .push(<&PathComponent>::try_from(component).unwrap())),
                    &format!("/{component}/{component}"),
                );
            }
            fn test_special_character(character: char) {
                if character != '.' {
                    test_special_component(&format!("{character}"));
                    test_special_component(&format!("{character}{character}"));
                }
                test_special_component(&format!("foo{character}"));
                test_special_component(&format!("foo{character}bar"));
                test_special_component(&format!("{character}bar"));
            }
            for character in "`~!@#$%^&*()-_=+[{]}|;:'\",<.>? ".chars() {
                test_special_character(character);
            }

            // And test some non-ascii utf8 characters
            for character in "äöü☕🍕".chars() {
                test_special_character(character);
            }
        }

        #[test]
        fn non_utf8() {
            // [non-utf8]
            test_non_utf8(b"\xFF");
            test_non_utf8(b"foo\xFF");
            test_non_utf8(b"foo\xFFbar");
            test_non_utf8(b"\xFFbar");

            // /[non-utf8]
            test_non_utf8(b"/\xFF");
            test_non_utf8(b"/foo\xFF");
            test_non_utf8(b"/foo\xFFbar");
            test_non_utf8(b"/\xFFbar");

            // /foo/[non-utf8]
            test_non_utf8(b"/foo/\xFF");
            test_non_utf8(b"/foo/foo\xFF");
            test_non_utf8(b"/foo/foo\xFFbar");
            test_non_utf8(b"/foo/\xFFbar");

            // /foo/[non-utf8]/bar
            test_non_utf8(b"/foo/\xFF/bar");
            test_non_utf8(b"/foo/foo\xFF/bar");
            test_non_utf8(b"/foo/foo\xFFbar/bar");
            test_non_utf8(b"/foo/\xFFbar/bar");

            // /[non-utf8]/bar
            test_non_utf8(b"/\xFF/bar");
            test_non_utf8(b"/foo\xFF/bar");
            test_non_utf8(b"/foo\xFFbar/bar");
            test_non_utf8(b"/\xFFbar/bar");
        }

        #[test]
        fn nullbyte() {
            // [with_nullbyte]
            test(Err(ParsePathError::InvalidFormat), "\0");
            test(Err(ParsePathError::InvalidFormat), "foo\0");
            test(Err(ParsePathError::InvalidFormat), "foo\0bar");
            test(Err(ParsePathError::InvalidFormat), "\0bar");

            // /[with_nullbyte]
            test(Err(ParsePathError::InvalidFormat), "/\0");
            test(Err(ParsePathError::InvalidFormat), "/foo\0");
            test(Err(ParsePathError::InvalidFormat), "/foo\0bar");
            test(Err(ParsePathError::InvalidFormat), "/\0bar");

            // /foo/[with_nullbyte]
            test(Err(ParsePathError::InvalidFormat), "/foo/\0");
            test(Err(ParsePathError::InvalidFormat), "/foo/foo\0");
            test(Err(ParsePathError::InvalidFormat), "/foo/foo\0bar");
            test(Err(ParsePathError::InvalidFormat), "/foo/\0bar");

            // /foo/[with_nullbyte]/bar
            test(Err(ParsePathError::InvalidFormat), "/foo/\0/bar");
            test(Err(ParsePathError::InvalidFormat), "/foo/foo\0/bar");
            test(Err(ParsePathError::InvalidFormat), "/foo/foo\0bar/bar");
            test(Err(ParsePathError::InvalidFormat), "/foo/\0bar/bar");

            // /[with_nullbyte]/bar
            test(Err(ParsePathError::InvalidFormat), "/\0/bar");
            test(Err(ParsePathError::InvalidFormat), "/foo\0/bar");
            test(Err(ParsePathError::InvalidFormat), "/foo\0bar/bar");
            test(Err(ParsePathError::InvalidFormat), "/\0bar/bar");
        }
    }

    mod split_last {
        use super::*;

        #[test]
        fn root_dir() {
            let path = AbsolutePath::try_from_str("/").unwrap();
            assert_eq!(None, path.split_last());
        }

        #[test]
        fn single_component() {
            let path = AbsolutePath::try_from_str("/foo").unwrap();
            assert_eq!(
                Some((
                    AbsolutePath::try_from_str("/").unwrap(),
                    "foo".try_into().unwrap(),
                )),
                path.split_last(),
            );
        }

        #[test]
        fn two_components() {
            let path = AbsolutePath::try_from_str("/foo/bar").unwrap();
            assert_eq!(
                Some((
                    AbsolutePath::try_from_str("/foo").unwrap(),
                    "bar".try_into().unwrap(),
                )),
                path.split_last(),
            );
        }

        #[test]
        fn three_components() {
            let path = AbsolutePath::try_from_str("/foo/bar/baz").unwrap();
            assert_eq!(
                Some((
                    AbsolutePath::try_from_str("/foo/bar").unwrap(),
                    "baz".try_into().unwrap(),
                )),
                path.split_last(),
            );
        }
    }

    mod iter {
        use super::*;

        // More iterator tests are in [super::iter]

        #[test]
        fn absolutepath_root_dir() {
            let path = AbsolutePath::try_from_str("/").unwrap();
            assert_eq!(
                Vec::<&PathComponent>::new(),
                path.iter().collect::<Vec<_>>(),
            );
        }

        #[test]
        fn absolutepath_single_component() {
            let path = AbsolutePath::try_from_str("/foo").unwrap();
            assert_eq!(
                vec![PathComponent::try_from_str("foo").unwrap(),],
                path.iter().collect::<Vec<_>>()
            );
        }

        #[test]
        fn absolutepath_two_components() {
            let path = AbsolutePath::try_from_str("/foo/bar").unwrap();
            assert_eq!(
                vec![
                    PathComponent::try_from_str("foo").unwrap(),
                    PathComponent::try_from_str("bar").unwrap(),
                ],
                path.iter().collect::<Vec<_>>()
            );
        }

        #[test]
        fn absolutepathbuf_root_dir() {
            let path = AbsolutePathBuf::try_from_string("/".to_string()).unwrap();
            assert_eq!(
                Vec::<&PathComponent>::new(),
                path.iter().collect::<Vec<_>>(),
            );
        }

        #[test]
        fn absolutepathbuf_single_component() {
            let path = AbsolutePathBuf::try_from_string("/foo".to_string()).unwrap();
            assert_eq!(
                vec![PathComponent::try_from_str("foo").unwrap(),],
                path.iter().collect::<Vec<_>>()
            );
        }

        #[test]
        fn absolutepathbuf_two_components() {
            let path = AbsolutePathBuf::try_from_string("/foo/bar".to_string()).unwrap();
            assert_eq!(
                vec![
                    PathComponent::try_from_str("foo").unwrap(),
                    PathComponent::try_from_str("bar").unwrap(),
                ],
                path.iter().collect::<Vec<_>>()
            );
        }
    }

    mod into_iter {
        use super::*;

        // More iterator tests are in [super::iter]

        #[test]
        fn absolutepath_root_dir() {
            let path = AbsolutePath::try_from_str("/").unwrap();
            assert_eq!(
                Vec::<&PathComponent>::new(),
                path.iter().collect::<Vec<_>>(),
            );
        }

        #[test]
        fn absolutepath_single_component() {
            let path = AbsolutePath::try_from_str("/foo").unwrap();
            assert_eq!(
                vec![PathComponent::try_from_str("foo").unwrap(),],
                path.into_iter().collect::<Vec<_>>()
            );
        }

        #[test]
        fn absolutepath_two_components() {
            let path = AbsolutePath::try_from_str("/foo/bar").unwrap();
            assert_eq!(
                vec![
                    PathComponent::try_from_str("foo").unwrap(),
                    PathComponent::try_from_str("bar").unwrap(),
                ],
                path.into_iter().collect::<Vec<_>>()
            );
        }

        #[test]
        fn absolutepathbuf_root_dir() {
            let path = AbsolutePathBuf::try_from_string("/".to_string()).unwrap();
            assert_eq!(
                Vec::<&PathComponent>::new(),
                path.iter().collect::<Vec<_>>(),
            );
        }

        #[test]
        fn absolutepathbuf_single_component() {
            let path = AbsolutePathBuf::try_from_string("/foo".to_string()).unwrap();
            assert_eq!(
                vec![PathComponent::try_from_str("foo").unwrap(),],
                path.into_iter().collect::<Vec<_>>()
            );
        }

        #[test]
        fn absolutepathbuf_two_components() {
            let path = AbsolutePathBuf::try_from_string("/foo/bar".to_string()).unwrap();
            assert_eq!(
                vec![
                    PathComponent::try_from_str("foo").unwrap(),
                    PathComponent::try_from_str("bar").unwrap(),
                ],
                path.into_iter().collect::<Vec<_>>()
            );
        }
    }
}
