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

#[test]
fn partitions_share_storage_and_rejoin_without_element_clones(){
    #[derive(Debug)] struct Counted(std::rc::Rc<std::cell::Cell<u32>>);
    impl Clone for Counted{fn clone(&self)->Self{self.0.set(self.0.get()+1);Self(self.0.clone())}}
    let count=std::rc::Rc::new(std::cell::Cell::new(0));
    let xs=(0..128).map(|_|Counted(count.clone())).collect::<Vec<_>>();
    let pointer=xs.as_ptr();let (a,b)=OwnedBuffer::new(xs).split(17);
    let (b,c)=b.split(33);let result=a.append(b).append(c).freeze();
    assert_eq!(result.len(),128);assert_eq!(result.as_ptr(),pointer);assert_eq!(count.get(),0);
    let shared=OwnedBuffer::shared(result);let alias=shared.clone();assert_eq!(count.get(),0);drop(alias);
    let result=shared.freeze();assert_eq!(result.as_ptr(),pointer);assert_eq!(count.get(),0);
}
#[test]
fn partitions_bound_writes_compaction_and_growth(){
    let(a,b)=OwnedBuffer::new(vec![1u32,2,3,4]).split(2);
    let a=a.set(2,99).push(7);let b=b.set(0,9).swap(0,1);
    assert_eq!(a.append(b).freeze(),vec![1,2,7,4,9]);
    let(a,b)=OwnedBuffer::new(vec![1u32,2,3,4,5,6]).split(3);
    assert_eq!(a.filter_inplace(|x|x%2==1).append(b.truncate(2)).freeze(),vec![1,3,4,5]);
    for at in [0,3,u32::MAX]{let(a,b)=OwnedBuffer::new(vec![1u32,2,3]).split(at);assert_eq!(a.append(b).freeze(),vec![1,2,3]);}
    let(a,b)=OwnedBuffer::new(Vec::<u32>::new()).split(0);assert!(a.append(b).freeze().is_empty());
}

#[test]
fn dynamic_full_owner_growth_does_not_clone_previous_elements(){
 #[derive(Debug)]struct Item(std::rc::Rc<std::cell::Cell<u32>>);
 impl Clone for Item{fn clone(&self)->Self{self.0.set(self.0.get()+1);Self(self.0.clone())}}
 let n=std::rc::Rc::new(std::cell::Cell::new(0));let mut owner=OwnedBuffer::shared(vec![]);
 let mut handles=vec![];
 for _ in 0..128{handles.push(owner.clone());owner=owner.push(Item(n.clone()));}
 assert_eq!(owner.len(),128);assert_eq!(n.get(),0);drop(handles);assert_eq!(owner.freeze().len(),128);assert_eq!(n.get(),0);
 let map=OwnedMap::shared().set(7u32,Item(n.clone()));let handle=map.clone();assert_eq!(n.get(),0);assert!(handle.contains(&7));
}

#[test]
fn affine_handle_release_transfers_full_storage_and_preserves_empty_views(){
 #[derive(Debug)]struct Item(std::rc::Rc<std::cell::Cell<u32>>);
 impl Clone for Item{fn clone(&self)->Self{self.0.set(self.0.get()+1);Self(self.0.clone())}}
 let n=std::rc::Rc::new(std::cell::Cell::new(0));let owner=OwnedBuffer::shared((0..16).map(|_|Item(n.clone())).collect());
 let dead_handle=owner.clone();let xs=owner.freeze_handle();assert_eq!(xs.len(),16);assert_eq!(n.get(),0);drop(dead_handle);
 let(a,b)=OwnedBuffer::shared(vec![1u32,2,3]).split(u32::MAX);
 assert_eq!(a.freeze_handle(),vec![1,2,3]);assert_eq!(b.get(0),None);
 assert_eq!(b.set(0,99).push(7).freeze(),vec![7]);
}

#[test]
fn bulk_ranges_preserve_allocation_overlap_and_scoped_boundaries(){
 let xs=vec![0u32,1,2,3,4,5,6];let pointer=xs.as_ptr();
 let(a,b)=OwnedBuffer::new(xs).split(3);
 let result=a.fill_range(1,u32::MAX,9).append(b.copy_within(0,1,u32::MAX)).freeze();
 assert_eq!(result,vec![0,9,9,3,3,4,5]);assert_eq!(result.as_ptr(),pointer);
 let xs=vec![0u32,1,2,3,4];let pointer=xs.as_ptr();
 let result=OwnedBuffer::new(xs).copy_within(0,1,4).copy_within(1,0,4).freeze();
 assert_eq!(result,vec![0,1,2,3,3]);assert_eq!(result.as_ptr(),pointer);
 assert_eq!(OwnedBuffer::new(vec!["a".to_owned(),"b".into(),"c".into()]).copy_within(0,1,2).freeze(),vec!["a","a","b"]);
 assert_eq!(OwnedBuffer::new(vec![u64::MAX,0]).fill_range(u32::MAX,u32::MAX,1).freeze(),vec![u64::MAX,0]);
}
