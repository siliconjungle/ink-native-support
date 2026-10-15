#![allow(dead_code)]
include!("../src/owned-values.rs.txt");
include!("../src/numeric-int.rs.txt");

#[test]
fn binary64_rounding_payloads_and_integer_boundaries(){
    let a=std::hint::black_box(f64::from_bits(0x3ff0000000000001));
    let b=std::hint::black_box(f64::from_bits(0x3fefffffffffffff));
    assert_ne!(a.mul_add(b,-1.0).to_bits(),0);
    assert_eq!(ink_f64(ink_f64(a*b)-1.0).to_bits(),0);
    assert_eq!(ink_f64(f64::from_bits(0x7ff0000000001234)).to_bits(),0x7ff8000000000000);
    assert_eq!(ink_f64(-0.0).to_bits(),1<<63);
    assert_eq!(ink_u64_from_f64(18446744073709551616.0),None);
    assert_eq!(ink_u64_from_f64(-0.5),Some(0));
    assert_eq!(ink_i32_from_f64(-2147483648.9),Some(i32::MIN));
    assert_eq!(ink_int_from_f64(f64::INFINITY),None);
    assert_eq!(ink_int_from_f64(-0.5),Some(0.into()));
    for (integer,bits)in [("9007199254740993",0x4340000000000000),("9007199254740995",0x4340000000000002),("-9007199254740995",0xc340000000000002),("18446744073709551615",0x43f0000000000000)]{
        assert_eq!(ink_f64_from_int(&integer.parse().unwrap()).to_bits(),bits);
    }
    for exponent in [0,1,52,53,54,1022,1023]{
        let n=num_bigint::BigInt::from(1u32)<<exponent;
        assert_eq!(ink_int_from_f64(ink_f64_from_int(&n)),Some(n));
    }
    assert!(ink_f64_from_int(&(num_bigint::BigInt::from(1u32)<<1024)).is_infinite());
    assert_eq!(ink_round_ties_even_f64(-0.5).to_bits(),1<<63);
    assert_eq!(ink_sqrt_f64(-1.0).to_bits(),0x7ff8000000000000);
}

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
