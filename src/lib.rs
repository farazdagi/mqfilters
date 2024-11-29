pub mod error;
pub use error::{QueryFilterError, QueryFilterResult};

#[cfg(feature = "bf")]
pub mod bf;

use std::{borrow::Borrow, hash::Hash};

#[cfg(feature = "bf")]
pub use bf::BloomFilter;

/// Defines membership query filter.
///
/// Each implementation of `QueryFilter` defines a filter that can be used to
/// query whether an element is a member of a set or not. False positives are
/// allowed (although considerable effort should be made to minimize them).
/// False negatives are not allowed.
pub trait QueryFilter<K> {
    /// Returns `true` if the value is believed to be in the filter.
    ///
    /// The value may be any borrowed form of the filter's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the inserted key type.
    fn contains<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized;
}

/// Defines a filter that supports adding elements.
pub trait InsertableQueryFilter<K>: QueryFilter<K> {
    /// Inserts an element into the filter.
    fn insert(&mut self, key: K)
    where
        K: Eq + Hash;
}

/// Defines a filter that supports removal of elements.
pub trait RemovableQueryFilter<K>: QueryFilter<K> {
    /// Removes an element from the filter.
    fn remove<Q>(&mut self, key: &Q)
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized;
}

/// Defines a filter that supports clearing all elements.
pub trait ClearableQueryFilter<K>: QueryFilter<K> {
    /// Removes all elements from the filter.
    fn clear(&mut self);
}
