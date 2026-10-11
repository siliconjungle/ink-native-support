#[allow(dead_code, unused_imports)]
mod generated {
    #[derive(Clone, Debug)]
    pub enum EventData {}
    impl EventData {
        fn wire(&self) -> (&str, serde_json::Value) {
            match *self {}
        }
    }
    include!("../src/typed-values.rs.txt");
    include!("../src/owned-values.rs.txt");
}
use generated::*;
use serde_json::json;
#[test]
fn shared_typed_support_preserves_wire_bits_and_owned_values() {
    for bits in [0u32, 0x80000000, 0x7fc12345, 0x7f800000, 0xff800000] {
        let f = f32::from_json(&json!({"F32Bits":bits})).unwrap();
        let again = f32::from_json(&f.to_json()).unwrap();
        assert_eq!(again.to_bits(), bits);
    }
    assert_eq!(
        i32::from_json(&json!(-2147483648i64)).unwrap().to_json(),
        json!(-2147483648i64)
    );
    assert!(u32::from_json(&json!(-1)).is_err());
    assert_eq!(
        OwnedBuffer::new(vec![1u32, 2, 3]).set(1, 9).freeze(),
        vec![1, 9, 3]
    );
    assert_eq!(OwnedBuffer::new(vec![u32::MAX,1,2]).scan_inplace(0u32,u32::wrapping_add).freeze(),vec![u32::MAX,0,2]);
    assert_eq!(OwnedBuffer::new(vec![4u32,4,4]).map_indexed_inplace(|i,x|i+x).freeze(),vec![4,5,6]);
    let value =
        <Vec<Option<u64>>>::from_json(&json!([{"Some":18446744073709551615u64},{"None":null}]))
            .unwrap();
    assert_eq!(
        value.to_json(),
        json!([{"Some":18446744073709551615u64},{"None":null}])
    );
}

#[test]
fn stable_owned_compaction(){assert_eq!(OwnedBuffer::new(vec![1u32,9,2,4]).filter_inplace(|x|x>2).freeze(),vec![9,4]);assert!(OwnedBuffer::new(Vec::<u32>::new()).filter_inplace(|_|true).freeze().is_empty());}
