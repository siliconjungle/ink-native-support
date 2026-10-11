include!("../src/indexed-table.rs.txt");
#[test]
fn arbitrary_slot_moves_and_rollback_match_an_independent_ordered_map() {
    let mut actual = InkTable::new();
    let mut expected = std::collections::BTreeMap::new();
    let mut seed = 17u64;
    for _ in 0..20_000 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let key = seed % 4000;
        if seed & 3 == 0 {
            assert_eq!(actual.remove(&key), expected.remove(&key));
        } else {
            assert_eq!(actual.insert(key, seed), expected.insert(key, seed));
        }
        assert_eq!(actual.len(), expected.len());
        assert_eq!(actual.get(&key), expected.get(&key));
        if seed % 97 == 0 {
            assert_eq!(
                actual.iter().collect::<Vec<_>>(),
                expected.iter().collect::<Vec<_>>()
            );
        }
    }
    let before = actual.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>();
    let mut undo = vec![];
    for key in 0..4000 {
        let old = if key % 2 == 0 {
            actual.remove(&key)
        } else {
            actual.insert(key, key)
        };
        undo.push((key, old));
    }
    for (key, old) in undo.into_iter().rev() {
        match old {
            Some(v) => {
                actual.insert(key, v);
            }
            None => {
                actual.remove(&key);
            }
        }
    }
    assert_eq!(
        actual.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
        before
    );
    let mut dense = actual.dense_values().copied().collect::<Vec<_>>();
    dense.sort();
    let mut ordered = actual.values().copied().collect::<Vec<_>>();
    ordered.sort();
    assert_eq!(dense, ordered);
}
