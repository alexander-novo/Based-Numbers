use delegate::delegate;
use tinyvec::TinyVec;

#[derive(Clone, Debug, Default)]
pub struct TinyMap<K: Default, V: Default, const N: usize> {
    inner: TinyVec<[(K, V); N]>,
}

impl<K: Default, V: Default, const N: usize> TinyMap<K, V, N> {
    pub fn new() -> Self {
        Self {
            inner: TinyVec::new(),
        }
    }

    delegate! {
        to self.inner {
            pub fn capacity(&self) -> usize;
            pub fn clear(&mut self);
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
            pub fn shrink_to_fit(&mut self);
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.iter().map(|(_, v)| v)
    }
}

impl<K: Default + Ord, V: Default, const N: usize> TinyMap<K, V, N> {
    pub fn entry(&mut self, key: K) -> TinyMapEntry<K, V, N> {
        match self.inner.binary_search_by_key(&&key, |(key, _)| key) {
            Ok(i) => TinyMapEntry::Occupied(&mut self.inner[i]),
            Err(i) => TinyMapEntry::Vacant(&mut self.inner, key, i),
        }
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        match self.inner.binary_search_by_key(&&key, |(key, _)| key) {
            Ok(i) => Some(std::mem::replace(&mut self.inner[i].1, val)),
            Err(i) => {
                self.inner.insert(i, (key, val));
                None
            }
        }
    }
}

impl<K: Default, V: Default, const N: usize> Extend<(K, V)> for TinyMap<K, V, N> {
    delegate! {
        to self.inner {
            fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T);
        }
    }
}

pub enum TinyMapEntry<'a, K: Default, V: Default, const N: usize> {
    Occupied(&'a mut (K, V)),
    Vacant(&'a mut TinyVec<[(K, V); N]>, K, usize),
}

impl<'a, K: Default, V: Default, const N: usize> TinyMapEntry<'a, K, V, N> {
    pub fn and_modify(mut self, f: impl FnOnce(&mut V)) -> Self {
        if let Self::Occupied(entry) = &mut self {
            f(&mut entry.1);
        }
        self
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            TinyMapEntry::Occupied(entry) => &mut entry.1,
            TinyMapEntry::Vacant(inner, key, idx) => {
                inner.insert(idx, (key, default));
                &mut inner[idx].1
            }
        }
    }
}
