#![allow(dead_code)]
include!("../src/owned-values.rs.txt");
#[test]
fn direct_push_preserves_owned_elements_across_growth_and_zero_sized_limit(){
    #[derive(Clone)]
    struct Tracked(std::rc::Rc<std::cell::Cell<usize>>,usize);
    impl Drop for Tracked {fn drop(&mut self){self.0.set(self.0.get()+1);}}
    let drops=std::rc::Rc::new(std::cell::Cell::new(0));
    let mut owner=DirectBuffer::new(Vec::with_capacity(3));
    let initial=owner.0.as_ptr();
    for i in 0..257{
        owner=owner.push(Tracked(drops.clone(),i));
        if i<3{assert_eq!(owner.0.as_ptr(),initial);}
        assert_eq!(drops.get(),0);
    }
    let values=owner.freeze();
    assert_eq!(values.iter().map(|v|v.1).collect::<Vec<_>>(),(0..257).collect::<Vec<_>>());
    drop(values);assert_eq!(drops.get(),257);
    // Zero-sized elements exercise the real u32 length limit without allocating
    // gigabytes or changing the production limit for the test.
    let edge=DirectBuffer::new(vec![();u32::MAX as usize-1]).push(());
    assert_eq!(edge.len(),u32::MAX);
    assert!(std::panic::catch_unwind(||edge.push(())).is_err());
}
#[test]
fn scoped_views_share_payloads_and_clip_nested_ranges_without_changing_owners() {
    let input=vec![String::from("a"),String::from("b"),String::from("c")];
    let direct=DirectBuffer::new(input.clone());
    let pointer=direct.0.as_ptr();
    let result=direct.with_range(1,u32::MAX,|view| {
        assert_eq!(view.0.as_ptr(),unsafe{pointer.add(1)});
        assert_eq!(view.len(),2);assert_eq!(view.get(0).as_deref(),Some("b"));
        view.with_range(1,1,|inner|inner.get(0))
    });
    assert_eq!(result.as_deref(),Some("c"));assert_eq!(direct.freeze(),input);
    let owner=OwnedBuffer::new(vec![1,2,3,4]);
    let (left,right)=owner.split(2);
    assert_eq!(right.with_range(1,u32::MAX,|view|view.get(0)),Some(4));
    assert_eq!(left.with_range(u32::MAX,u32::MAX,|view|view.len()),0);
    assert_eq!(left.append(right).freeze(),vec![1,2,3,4]);
    let owner=OwnedBuffer::shared(vec![1,2,3]);
    let view=owner.range_handle(1,1);
    assert!(matches!((&owner.0,&view.0),(InkBufferStorage::Shared{data:a,..},InkBufferStorage::Shared{data:b,..})if std::rc::Rc::ptr_eq(a,b)));
    assert_eq!(view.get(0),Some(2));drop(view);
    assert_eq!(owner.push(4).freeze_handle(),vec![1,2,3,4]);
}
#[test]
fn direct_owner_bulk_edits_match_snapshot_oracle_and_partition_order() {
    for len in [0, 1, 2, 63, 64, 65, 257] {
        for a in [0, 1, 64, u32::MAX] {
            for b in [0, 1, 63, u32::MAX] {
                for n in [0, 1, 65, u32::MAX] {
                    let xs = (0..len).map(|i| i as u32).collect::<Vec<_>>();
                    let mut oracle = xs.clone();
                    let s = (a as usize).min(len);
                    let d = (b as usize).min(len);
                    let count = (n as usize).min(len - s).min(len - d);
                    for i in 0..count {
                        oracle[d + i] = xs[s + i];
                    }
                    let actual = DirectBuffer::new(xs).copy_within(a, b, n);
                    let (left, right) = actual.split(a);
                    assert_eq!(left.append(right).freeze(), oracle);
                    let before_fill = oracle.clone();
                    let s = (b as usize).min(len);
                    for v in &mut oracle[s..s + (n as usize).min(len - s)] {
                        *v = u32::MAX;
                    }
                    assert_eq!(
                        DirectBuffer::new(before_fill)
                            .fill_range(b, n, u32::MAX)
                            .freeze(),
                        oracle
                    );
                }
            }
        }
    }
    let xs = vec![1, 2, 3, 4];
    let owner = DirectBuffer::new(xs.clone());
    let snapshot = owner.clone();
    assert_eq!(owner.map_inplace(|v| v + 10).freeze(), vec![11, 12, 13, 14]);
    assert_eq!(snapshot.freeze(), xs);
}
#[test]
fn packed_words_preserve_all_bits_through_push_set_and_promotion() {
    fn exercise<const B: u8, const BASE: u8>() {
        let values = [
            0,
            1,
            255,
            256,
            65535,
            65536,
            u32::MAX as u64,
            u32::MAX as u64 + 1,
            u64::MAX,
        ];
        let mut packed = PackedWords::<B, BASE>::default();
        for (i, v) in values.into_iter().enumerate() {
            packed.push(v);
            for j in 0..=i {
                assert_eq!(packed.get(j), values[j]);
            }
        }
        let snapshot = packed.clone();
        packed.set(0, u64::MAX);
        assert_eq!(packed.get(0), u64::MAX);
        assert_eq!(snapshot.get(0), 0);
        packed.truncate(3);
        assert_eq!(packed.len(), 3);
    }
    exercise::<8, 32>();
    exercise::<16, 32>();
    exercise::<32, 32>();
    exercise::<8, 64>();
    exercise::<16, 64>();
    exercise::<32, 64>();
    exercise::<64, 64>();
    let mut narrow = PackedWords::<8, 32>::default();
    for i in 0..200 {
        narrow.push(i);
    }
    assert_eq!(narrow.payload_bytes(), 200);
    narrow.push(256);
    assert_eq!(narrow.payload_bytes(), 201 * 4);
}
#[test]
fn packed_bits_preserve_boundary_bits_and_clear_truncated_padding() {
    let mut bits = PackedBits::default();
    for i in 0..257 {
        bits.push(i % 3 == 0);
    }
    for i in 0..257 {
        assert_eq!(bits.get(i), i % 3 == 0);
    }
    for n in [256, 129, 128, 65, 64, 63, 1, 0] {
        bits.truncate(n);
        bits.push(false);
        assert!(!bits.get(n));
        bits.truncate(n);
    }
    assert_eq!(bits.len(), 0);
    assert_eq!(bits.payload_bytes(), 0);
}

#[test]
fn capacity_failures_leave_packed_columns_readable() {
    let mut words=PackedWords::<8,32>::default();words.push(7);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||words.reserve(usize::MAX))).is_err());assert_eq!(words.get(0),7);
    let mut bits=PackedBits::default();bits.push(true);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||bits.reserve(usize::MAX))).is_err());assert!(bits.get(0));
    let mut other=PackedWords::<8,32>::default();other.push(65536);words.prepare_append(&other);assert_eq!(words.get(0),7);words.push(other.get(0));assert_eq!(words.get(1),65536);
}
