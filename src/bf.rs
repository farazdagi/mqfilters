use {
    crate::{ClearableQueryFilter, InsertableQueryFilter, QueryFilter},
    fixedbitset::FixedBitSet as BitSet,
    hash_iter::{DoubleHashHasher, HashIterHasher},
    std::{borrow::Borrow, hash::Hash, marker::PhantomData},
};

pub struct BloomFilter<K>
where
    K: Eq + Hash,
{
    bits: BitSet,
    hasher: DoubleHashHasher,
    k: usize,
    phantom: PhantomData<K>,
}

impl<K> BloomFilter<K>
where
    K: Eq + Hash,
{
    /// Creates a new Bloom filter with a desired capacity and false positive
    /// rate.
    pub fn new(capacity: usize, fp_rate: f64) -> Self {
        Self::with_capacity(capacity, fp_rate)
    }

    /// Creates a new Bloom filter with a desired size (in bytes) and false
    /// positive rate.
    pub fn with_size(size: usize, fp_rate: f64) -> Self {
        Self::with_size_and_hasher(size, fp_rate, DoubleHashHasher::new())
    }

    /// Creates a new Bloom filter with a desired capacity and false positive
    /// rate.
    pub fn with_capacity(capacity: usize, fp_rate: f64) -> Self {
        Self::with_capacity_and_hasher(capacity, fp_rate, DoubleHashHasher::new())
    }

    /// Creates a new Bloom filter with a desired size (in bytes), false
    /// positive, and hasher.
    pub fn with_size_and_hasher(size: usize, fp_rate: f64, hasher: DoubleHashHasher) -> Self {
        let capacity = optimal_capacity(size * 8, fp_rate);
        Self::with_capacity_and_hasher(capacity, fp_rate, hasher)
    }

    /// Creates a new Bloom filter with a desired capacity, false positive rate,
    /// and hasher.
    pub fn with_capacity_and_hasher(
        capacity: usize,
        fp_rate: f64,
        hasher: DoubleHashHasher,
    ) -> Self {
        let bit_count = optimal_bit_count(capacity, fp_rate);
        let k = optimal_hash_count(capacity, bit_count);
        Self {
            bits: BitSet::with_capacity(bit_count),
            hasher,
            k,
            phantom: PhantomData,
        }
    }

    /// Returns the approximate number of elements currently in the filter.
    pub fn approx_current_capacity(&self) -> usize {
        let bits_count = self.bits.len() as f64;
        let ones_count = self.bits.count_ones(..) as f64;
        let hash_count = self.k as f64;
        let count = -(bits_count / hash_count) * (1. - (ones_count / bits_count)).ln();

        count.round() as usize
    }
}

/// Given a capacity and a desired false positive rate, returns the optimal
/// number of bits to use (size of the filter, `m`), along with an for an
/// optimal `k`.
pub fn optimal_bit_count(capacity: usize, fp_rate: f64) -> usize {
    let ln2 = std::f64::consts::LN_2;
    let n = capacity as f64;
    let p = fp_rate;

    (-n * p.ln() / ln2.powi(2)).ceil() as usize
}

/// Given a desired false positive rate and the number of bits, returns the
/// optimal capacity (number of items hashed into filter, `n`).
pub fn optimal_capacity(bit_count: usize, fp_rate: f64) -> usize {
    let ln2 = std::f64::consts::LN_2;
    let m = bit_count as f64;
    let p = fp_rate;

    (m * ln2.powi(2) / -p.ln()).round() as usize
}

/// Returns the optimal number of hash functions to use (`k`).
///
/// Current implementation relies on double hashing, so for a given key, it
/// creates this many hash values (while internally using up to two
/// different hash functions -- mostly one).
pub fn optimal_hash_count(capacity: usize, bit_count: usize) -> usize {
    let ln2 = std::f64::consts::LN_2;
    let n = capacity as f64;
    let m = bit_count as f64;

    (m / n * ln2).ceil() as usize
}

impl<K> QueryFilter<K> for BloomFilter<K>
where
    K: Eq + Hash,
{
    fn contains<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        for hash in self.hasher.hash_iter(key, self.k) {
            let index = (hash % self.bits.len() as u64) as usize;
            if !self.bits.contains(index) {
                return false;
            }
        }
        true
    }
}

impl<K> InsertableQueryFilter<K> for BloomFilter<K>
where
    K: Eq + Hash,
{
    fn insert(&mut self, key: K) {
        for hash in self.hasher.hash_iter(&key, self.k) {
            let index = (hash % self.bits.len() as u64) as usize;
            self.bits.insert(index);
        }
    }
}

impl<K> ClearableQueryFilter<K> for BloomFilter<K>
where
    K: Eq + Hash,
{
    fn clear(&mut self) {
        self.bits.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimal_bit_count_works() {
        // Given `n` (capacity) and `p` (false positive rate), find `m` (size) and
        // optimal `k`.
        let test_cases = [
            (10, 0.05, 63, 5),
            (100, 0.05, 624, 5),
            (100, 0.1, 480, 4),
            (100, 0.01, 959, 7),
            (1000, 0.01, 9586, 7),
            (10000, 0.01, 95851, 7),
            (100000, 0.01, 958506, 7),
        ];
        for (n, p, m, k) in test_cases {
            assert_eq!(optimal_bit_count(n, p), m);
            assert_eq!(optimal_hash_count(n, m), k);
            assert_eq!(optimal_bit_count(optimal_capacity(m, p), p), m);
        }
    }

    #[test]
    fn optimal_capacity_works() {
        let test_cases = [
            (1 << 13 as usize, 0.01, 855, 7), // 1 KiB
            (1 << 13 as usize, 0.05, 1314, 5),
            (1 << 23 as usize, 0.01, 875175, 7), // 1 MiB
            (1 << 23 as usize, 0.05, 1345358, 5),
            (1 << 33 as usize, 0.01, 896179684, 7), // 1 GiB
            (1 << 33 as usize, 0.05, 1377646461, 5),
        ];
        for (m, p, n, k) in test_cases {
            assert_eq!(optimal_capacity(m, p), n);
            assert_eq!(optimal_hash_count(n, m), k);
            assert_eq!(optimal_capacity(optimal_bit_count(n, p), p), n);
        }
    }

    #[test]
    fn optimal_hash_count_works() {
        let test_cases = [
            (10, 63, 5),
            (100, 624, 5),
            (100, 480, 4),
            (100, 959, 7),
            (1000, 9586, 7),
            (10000, 95851, 7),
            (100000, 958506, 7),
        ];
        for (n, m, k) in test_cases {
            assert_eq!(optimal_hash_count(n, m), k);
        }
    }
}
