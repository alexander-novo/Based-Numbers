use delegate::delegate;
use tinyvec::TinyVec;

/// A binary tree map with backing storage of a [`TinyVec`].
#[derive(Clone, Debug, Default)]
pub struct TinyMap<K: Default, V: Default, const N: usize> {
    inner: TinyVec<[(K, V); N]>,
}

impl<K: Default, V: Default, const N: usize> TinyMap<K, V, N> {
    /// Creates a new empty [`TinyMap`].
    pub fn new() -> Self {
        Self {
            inner: TinyVec::new(),
        }
    }

    delegate! {
        to self.inner {
            /// The capacity of the internal backing storage.
            pub fn capacity(&self) -> usize;

            /// Remove all elements.
            pub fn clear(&mut self);

            /// Whether or not the map is empty.
            pub fn is_empty(&self) -> bool;

            /// The length of the map (in no. of elements)
            pub fn len(&self) -> usize;

            /// Shrink the capacity of the map as much as possible. This can
            /// cause the backing storage [`TinyVec`] to de-allocate and "inline"
            /// itself if the resulting capacity is less than or equal to `N`.
            pub fn shrink_to_fit(&mut self);
        }
    }

    /// An iterator over the values contained in the map.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.iter().map(|(_, v)| v)
    }
}

impl<K: Default + Ord, V: Default, const N: usize> TinyMap<K, V, N> {
    /// Returns a symbolic "entry" value corresponding to the given key,
    /// which enables in-place modification and/or delayed insertion of
    /// a new element at that key.
    pub fn entry(&mut self, key: K) -> TinyMapEntry<K, V, N> {
        match self.inner.binary_search_by_key(&&key, |(key, _)| key) {
            Ok(idx) => TinyMapEntry::Occupied(&mut self.inner[idx]),
            Err(idx) => TinyMapEntry::Vacant {
                inner: &mut self.inner,
                key,
                idx,
            },
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
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

/// A symbolic "entry" into a [`TinyMap`] at a specific key. Enables
/// in-place modification and delayed insertion of new values at that key.
pub enum TinyMapEntry<'a, K: Default, V: Default, const N: usize> {
    /// If the key already exists in the map, this is a pointer to its place in the
    /// backing storage.
    Occupied(&'a mut (K, V)),
    /// Otherwise, keep track of where in the backing storage we should insert
    /// a new element, should we want to.
    Vacant {
        inner: &'a mut TinyVec<[(K, V); N]>,
        key: K,
        idx: usize,
    },
}

impl<'a, K: Default, V: Default, const N: usize> TinyMapEntry<'a, K, V, N> {
    /// Provides in-place mutable access to an occupied entry before any potential inserts into the map.
    pub fn and_modify(mut self, f: impl FnOnce(&mut V)) -> Self {
        if let Self::Occupied(entry) = &mut self {
            f(&mut entry.1);
        }
        self
    }

    /// Ensures a value is in the entry by inserting the default if empty, and returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            TinyMapEntry::Occupied(entry) => &mut entry.1,
            TinyMapEntry::Vacant { inner, key, idx } => {
                inner.insert(idx, (key, default));
                &mut inner[idx].1
            }
        }
    }
}
