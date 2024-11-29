# Approximate Membership Query Filters (`mqfilters`)

Highly optimized approximate membership query filters in safe Rust.

The purpose of this repository is to gather multiple implementations of approximate membership query
algorithms, along with a brief discussion of their applicability in different contexts. The crust of
the problem is to be able to provide a data structure that will take a fraction of space compared to
the original data, while still being able to answer the question whether a given element is in the
set or not.

## Implemented Filters

- [x] Classic Bloom Filter ([`bf`](src/bf.rs))

### Classic Bloom Filter (`bf`)

Based on the original Bloom Filter described by Burton Howard Bloom in the seminal
[Space/Time Trade-offs in Hash Coding with Allowable Errors, 1970](https://dl.acm.org/doi/pdf/10.1145/362686.362692)
paper.

Bloom considered three factors when optimizing the membership queries:

- reject time (an average time to classify an element as absent)
- space (memory usage)
- probability of false positives

The main idea is that by introducing a small probability of false positives, we can significantly
reduce the space usage, all without increasing the reject time. This reduction in space can be very
significant as it can become a deciding factor in whether a filter can be stored in memory or not.

The original design is only slightly updated, to use double hashing (instead of multiple distinct
hash functions), as described in
[Less Hashing, Same Performance: Building a Better Bloom Filter, 2006](https://www.eecs.harvard.edu/~michaelm/postscripts/rsa2008.pdf)
by Kirsch and Mitzenmacher.

#### Variants and Future work

Currently implemented is a semi-dynamic Bloom Filter, which means that it supports insertions, but
not deletions, so filter can be grow, but not shrink.

- Static Bloom Filter: once created, it cannot be modified (just recreated). This can be useful in
  scenarios where the filter is created once and then queried multiple times. The obvious example is
  `SSTables` in `LSMTrees`, where we have to merge data from time to time and we can create a new
  filter during such merge. Having this invariant allows for a more efficient implementation.
- Dynamic Bloom Filter: supports deletions. This is often implemented as counting Bloom Filter.
