#![allow(dead_code)]
include!("../src/owned-values.rs.txt");

#[inline(never)]
fn rounded_product(a:f32,b:f32)->f32 {
    let p=a*b;
    if p.is_nan(){f32::from_bits(0x7fc00000)}else{p}
}
#[inline(never)]
fn rounded_sum(a:f32,b:f32)->f32 {
    let p=a+b;
    if p.is_nan(){f32::from_bits(0x7fc00000)}else{p}
}

#[test]
fn inlined_canonicalisation_preserves_each_arithmetic_rounding_point() {
    // FMA would produce a nonzero result; two binary32 operations produce zero.
    let a=std::hint::black_box(f32::from_bits(0x3f800001));
    let b=std::hint::black_box(f32::from_bits(0x3f7fffff));
    assert_ne!(a.mul_add(b,-1.0).to_bits(),0);
    assert_eq!(ink_f32(ink_f32(a*b)+(-1.0)).to_bits(),0);

    let special=[0,0x80000000,1,0x80000001,0x007fffff,0x00800000,
        0x3f800001,0x3f7fffff,0x7f7fffff,0xff7fffff,0x7f800000,
        0xff800000,0x7fc12345,0x7fa12345,0xff812345];
    for x in special {for y in special {for z in special {
        let (a,b,c)=(f32::from_bits(x),f32::from_bits(y),f32::from_bits(z));
        let actual=ink_f32(ink_f32(a*b)+c).to_bits();
        let expected=rounded_sum(rounded_product(a,b),c).to_bits();
        assert_eq!(actual,expected,"{x:08x} {y:08x} {z:08x}");
    }}}
    let mut seed=17u32;
    let mut next=||{seed=seed.wrapping_mul(1664525).wrapping_add(1013904223);f32::from_bits(seed)};
    for _ in 0..100_000 {
        let (a,b,c)=(next(),next(),next());
        assert_eq!(ink_f32(ink_f32(a*b)+c).to_bits(),rounded_sum(rounded_product(a,b),c).to_bits());
    }
    for bits in [0,0x80000000,1,0x80000001,0x007fffff,0x00800000,0x7f800000,0xff800000] {
        assert_eq!(ink_f32(std::hint::black_box(f32::from_bits(bits))).to_bits(),bits);
    }
}
