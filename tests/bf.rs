use mqfilters::{BloomFilter, ClearableQueryFilter, InsertableQueryFilter, QueryFilter};

#[test]
fn default_filter() {
    let mut filter = BloomFilter::new(100, 0.01);
    assert_eq!(filter.approx_current_capacity(), 0);
    assert!(!filter.contains(&"hello"));

    filter.insert("hello");
    assert!(filter.contains(&"hello"));
    assert_eq!(filter.approx_current_capacity(), 1);

    filter.insert("hello");
    filter.insert("hello");
    assert_eq!(filter.approx_current_capacity(), 1);

    filter.clear();
    assert!(!filter.contains(&"hello"));
    assert_eq!(filter.approx_current_capacity(), 0);
}

#[test]
fn with_size() {
    let fp_rate = 0.01;
    let size = 100 << 13;
    let mut filter = BloomFilter::with_size(size, fp_rate);

    let items_cnt = 500000;
    let mut fp_count = 0;
    for i in 0..items_cnt {
        if filter.contains(&i) {
            fp_count += 1;
        }
        filter.insert(i);
        // Ensure that no false negatives are present.
        assert!(filter.contains(&i));
    }

    assert!(items_cnt - filter.approx_current_capacity() < 100);
    assert!((fp_count as f64) < items_cnt as f64 * fp_rate as f64);
}

#[test]
fn with_capacity() {
    let fp_rate = 0.01;
    let capacity = 100000;
    let mut filter = BloomFilter::with_capacity(capacity, fp_rate);
    let mut fp_count = 0;
    for i in 0..capacity {
        if filter.contains(&i) {
            fp_count += 1;
        }
        filter.insert(i);
        // Ensure that no false negatives are present.
        assert!(filter.contains(&i));
    }

    assert!(capacity - filter.approx_current_capacity() < 100);
    assert!((fp_count as f64) < capacity as f64 * fp_rate as f64);
}
