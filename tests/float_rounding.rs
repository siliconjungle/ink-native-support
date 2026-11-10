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

/// Canonicalisation has to survive optimisation, where a compiler may fold a
/// NaN selection away, so this compiles the support text with `rustc -O` and
/// runs every float helper over NaN payloads, signed NaNs and the inputs
/// that produce a NaN.
#[test]
fn optimised_helpers_return_only_the_canonical_nan() {
    let dir = std::env::temp_dir().join(format!("ink-canonical-nan-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = format!(
        "#![allow(dead_code,unused)]\n{}\n{}",
        include_str!("../src/owned-values.rs.txt"),
        r#"
use std::hint::black_box;
fn main(){
    let f32s=[0x7fe4e7bfu32,0xffa12345,0x7f800001,0x7fc00000,0xffc00000,0xbf800000,0xff800000,0x7f800000,0x80000000,0,0x40800000,0xc0200000];
    let f64s=[0x7ff0000000001234u64,0xfff4000000000001,0x7ff8000000000000,0xfff8000000000000,0xbff0000000000000,0xfff0000000000000,0x7ff0000000000000,1<<63,0,0x4010000000000000];
    let mut bad=vec![];
    let mut see32=|name:&str,input:u32,v:f32|{if v.is_nan()&&v.to_bits()!=0x7fc00000{bad.push(format!("{name}({input:08x}) = {:08x}",v.to_bits()));}};
    for &b in &f32s{
        let x=black_box(f32::from_bits(b));
        see32("ink_f32",b,ink_f32(x));see32("ink_f32_carried",b,ink_f32_carried(x));
        see32("ink_sqrt",b,ink_sqrt(x));see32("ink_floor",b,ink_floor(x));see32("ink_ceil",b,ink_ceil(x));
        see32("ink_trunc",b,ink_trunc(x));see32("ink_round_ties_even",b,ink_round_ties_even(x));
        for &c in &f32s{
            let y=black_box(f32::from_bits(c));
            see32("ink_f32_div",b,ink_f32_div(x,y));see32("add",b,ink_f32(x+y));see32("mul",b,ink_f32(x*y));see32("sub",b,ink_f32(x-y));
            see32("carried add",b,ink_f32_carried(x+y));see32("carried mul",b,ink_f32_carried(x*y));
            // One canonicalisation after several operations.
            see32("nested",b,ink_f32((x*y)+(x-y)));see32("nested root",b,ink_sqrt((x*x)+(y*y)));see32("nested quotient",b,ink_f32_div(x+y,x*y));
        }
    }
    // Element-wise loops and reductions, where the helpers are inlined and may be vectorised.
    let xs:Vec<f32>=black_box(f32s.iter().cycle().take(96).map(|b|f32::from_bits(*b)).collect());
    for (i,v) in xs.iter().map(|x|ink_sqrt(*x)).collect::<Vec<f32>>().into_iter().enumerate(){see32("sqrt loop",f32s[i%f32s.len()],v);}
    for (i,v) in xs.iter().map(|x|ink_f32((*x*0.5)+1.0)).collect::<Vec<f32>>().into_iter().enumerate(){see32("map loop",f32s[i%f32s.len()],v);}
    for start in 0..f32s.len(){see32("sum",f32s[start],xs[start..].iter().fold(0.0f32,|a,b|ink_f32_carried(a+*b)));}
    let mut see64=|name:&str,input:u64,v:f64|{if v.is_nan()&&v.to_bits()!=0x7ff8000000000000{bad.push(format!("{name}({input:016x}) = {:016x}",v.to_bits()));}};
    for &b in &f64s{
        let x=black_box(f64::from_bits(b));
        see64("ink_f64",b,ink_f64(x));see64("ink_f64_carried",b,ink_f64_carried(x));
        see64("ink_sqrt_f64",b,ink_sqrt_f64(x));see64("ink_floor_f64",b,ink_floor_f64(x));see64("ink_ceil_f64",b,ink_ceil_f64(x));
        see64("ink_trunc_f64",b,ink_trunc_f64(x));see64("ink_round_ties_even_f64",b,ink_round_ties_even_f64(x));
        for &c in &f64s{
            let y=black_box(f64::from_bits(c));
            see64("ink_f64_div",b,ink_f64_div(x,y));see64("add",b,ink_f64(x+y));see64("mul",b,ink_f64(x*y));
            see64("carried add",b,ink_f64_carried(x+y));see64("nested root",b,ink_sqrt_f64((x*x)+(y*y)));
        }
    }
    assert!(bad.is_empty(),"non-canonical NaN results: {bad:?}");
    // Non-NaN results are untouched.
    assert_eq!(ink_sqrt(black_box(4.0)).to_bits(),2.0f32.to_bits());
    assert_eq!(ink_sqrt(black_box(-0.0)).to_bits(),0x80000000);
    assert_eq!(ink_sqrt_f64(black_box(-0.0)).to_bits(),1<<63);
    assert_eq!(ink_f32_carried(black_box(f32::NEG_INFINITY)).to_bits(),0xff800000);
}
"#
    );
    std::fs::write(dir.join("canonical.rs"), source).unwrap();
    let compiled = std::process::Command::new("rustc")
        .args(["--edition=2021", "-O"])
        .arg(dir.join("canonical.rs"))
        .arg("-o")
        .arg(dir.join("canonical"))
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    let run = std::process::Command::new(dir.join("canonical")).output().unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
}
