use lru::LruCache;
use std::hash::Hash;
use std::iter::{Iterator, FusedIterator};

// TODO Replace with https://github.com/jeromefroe/lru-rs/pull/129

pub struct LruCacheIterator<K, V>
where
    K: Hash + Eq,
{
    cache: LruCache<K, V>,
}

pub trait LruCacheIntoIter<K, V>
where
    K: Hash + Eq,
{
    fn into_iter(self) -> LruCacheIterator<K, V>;
}

impl<K, V> LruCacheIntoIter<K, V> for LruCache<K, V>
where
    K: Hash + Eq,
{
    fn into_iter(self) -> LruCacheIterator<K, V> {
        LruCacheIterator { cache: self }
    }
}

impl<K, V> Iterator for LruCacheIterator<K, V>
where
    K: Hash + Eq,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        self.cache.pop_lru()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.cache.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.cache.len()
    }
}

impl<K, V> ExactSizeIterator for LruCacheIterator<K, V> where K: Hash + Eq {}
impl<K, V> FusedIterator for LruCacheIterator<K, V> where K: Hash + Eq  {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_into_iter() {
        let mut cache = LruCache::new(3);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        let mut iter = cache.into_iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(("a", 1)));

        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(("b", 2)));

        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(("c", 3)));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }
}
