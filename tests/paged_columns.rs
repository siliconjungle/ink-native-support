#[allow(dead_code)]
mod runtime { include!("../src/paged-columns.rs.txt"); }
use runtime::PagedColumn;
use std::sync::atomic::{AtomicUsize,Ordering};
static COPIES:AtomicUsize=AtomicUsize::new(0);
#[derive(Debug,PartialEq)] struct Counted(u32);
impl Clone for Counted {fn clone(&self)->Self{COPIES.fetch_add(1,Ordering::Relaxed);Self(self.0)}}

#[test]
fn snapshots_share_pages_and_only_touched_pages_detach(){
 let mut values=PagedColumn::<Counted,256>::with_capacity(4096);
 for i in 0..4096{values.push(Counted(i));}
 COPIES.store(0,Ordering::Relaxed);
 let before=values.clone();assert_eq!(COPIES.load(Ordering::Relaxed),0);
 assert!(!values.set_if_changed_by(257,Counted(257),|a,b|a==b));
 assert_eq!(COPIES.load(Ordering::Relaxed),0);
 assert!(values.set_if_changed_by(257,Counted(999),|a,b|a==b));
 assert_eq!(COPIES.load(Ordering::Relaxed),256);
 unsafe{*values.get_unchecked_mut(258)=Counted(1000);}
 assert_eq!(COPIES.load(Ordering::Relaxed),256);
 assert_eq!(unsafe{before.get_unchecked(257)},&Counted(257));
 assert_eq!(unsafe{values.get_unchecked(257)},&Counted(999));
}

#[test]
fn bit_equality_keeps_nan_payloads_and_signed_zero(){
 let mut values=PagedColumn::<f32,16>::default();
 for bits in [0,0x80000000,0x7fc01234,0x7fc05678]{values.push(f32::from_bits(bits));}
 let before=values.clone();let identity=values.as_ptr();
 for (i,bits)in [0,0x80000000,0x7fc01234,0x7fc05678].into_iter().enumerate(){
  assert!(!values.set_if_changed_by(i,f32::from_bits(bits),|a,b|a.to_bits()==b.to_bits()));
 }
 assert_eq!(values.as_ptr(),identity);
 assert!(values.set_if_changed_by(0,-0.0,|a,b|a.to_bits()==b.to_bits()));
 assert!(values.set_if_changed_by(2,f32::from_bits(0x7fc05678),|a,b|a.to_bits()==b.to_bits()));
 for(i,bits)in [0,0x80000000,0x7fc01234,0x7fc05678].into_iter().enumerate(){assert_eq!(unsafe{before.get_unchecked(i)}.to_bits(),bits);}
 assert_eq!(unsafe{values.get_unchecked(0)}.to_bits(),0x80000000);
 assert_eq!(unsafe{values.get_unchecked(2)}.to_bits(),0x7fc05678);
}

#[test]
fn append_truncate_growth_and_shared_before_images_match_vectors(){
 for page in [16usize,256]{
  fn check<const P:usize>(){
   let mut actual=PagedColumn::<u64,P>::default();let mut want=Vec::new();
   let mut retained=Vec::new();let mut seed=932847u64;
   for step in 0..5000{
    seed=seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    match (seed>>32)%5{
     0 if !want.is_empty()=>{let i=seed as usize%want.len();unsafe{*actual.get_unchecked_mut(i)=seed;}want[i]=seed;},
     1=>{let n=seed as usize%(want.len()+1);actual.truncate(n);want.truncate(n);},
     2=>{let mut rhs=PagedColumn::<u64,P>::default();let n=seed as usize%(P*2+1);for i in 0..n{rhs.push(i as u64);}let old=rhs.clone();actual.append(&mut rhs);want.extend(0..n as u64);assert_eq!(rhs.len(),0);assert_eq!(old.len(),n);for i in 0..n{assert_eq!(unsafe{*old.get_unchecked(i)},i as u64);}},
     _=>{actual.push(seed);want.push(seed);},
    }
    if step%97==0{retained.push((actual.clone(),want.clone()));}
    assert_eq!(actual.len(),want.len());
    for (i,x) in want.iter().enumerate(){assert_eq!(unsafe{actual.get_unchecked(i)},x);}
   }
   for (old,expected)in retained{for(i,x)in expected.iter().enumerate(){assert_eq!(unsafe{old.get_unchecked(i)},x);}}
  }
  if page==16{check::<16>()}else{check::<256>()}
 }
}
