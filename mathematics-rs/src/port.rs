// A Rust port of FFmpeg n7.1 libavutil/mathematics.c, changed from the C original.
// Copyright (c) 2005-2012 Michael Niedermayer
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of the integer and Bessel functions of `mathematics.c`; `lib.rs` exports each
//! function under its C name.

#![forbid(unsafe_code)]

use std::ffi::c_int;

const AV_ROUND_NEAR_INF: c_int = 5;
const AV_ROUND_PASS_MINMAX: c_int = 8192;
const INT_MAX: i64 = i32::MAX as i64;

/// Port of `int64_t av_gcd(int64_t a, int64_t b)` (`mathematics.c:37`): binary GCD.
pub fn av_gcd(a: i64, b: i64) -> i64 {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    let za = a.trailing_zeros();
    let zb = b.trailing_zeros();
    let k = za.min(zb);
    let mut u = (a >> za).wrapping_abs();
    let mut v = (b >> zb).wrapping_abs();
    while u != v {
        if u > v {
            std::mem::swap(&mut u, &mut v);
        }
        v -= u;
        v >>= v.trailing_zeros();
    }
    ((u as u64) << k) as i64
}

/// Port of `int64_t av_rescale_rnd(int64_t a, int64_t b, int64_t c, enum AVRounding rnd)` (`mathematics.c:58`):
/// `a * b / c` rounded as `rnd` says, `INT64_MIN` on invalid arguments or overflow.
pub fn av_rescale_rnd(a: i64, b: i64, c: i64, rnd: c_int) -> i64 {
    let mode = rnd & !AV_ROUND_PASS_MINMAX;
    if c <= 0 || b < 0 || !((mode as u32) <= 5 && mode != 4) {
        return i64::MIN;
    }
    let mut rnd = rnd;
    if rnd & AV_ROUND_PASS_MINMAX != 0 {
        if a == i64::MIN || a == i64::MAX {
            return a;
        }
        rnd -= AV_ROUND_PASS_MINMAX;
    }
    if a < 0 {
        let flipped = rnd ^ ((rnd >> 1) & 1);
        return (av_rescale_rnd(-a.max(-i64::MAX), b, c, flipped) as u64).wrapping_neg() as i64;
    }
    let r = if rnd == AV_ROUND_NEAR_INF {
        c / 2
    } else if rnd & 1 != 0 {
        c - 1
    } else {
        0
    };
    if b <= INT_MAX && c <= INT_MAX {
        if a <= INT_MAX {
            return (a * b + r) / c;
        }
        let ad = a / c;
        let a2 = (a % c * b + r) / c;
        if ad >= INT_MAX && b != 0 && ad > (i64::MAX - a2) / b {
            return i64::MIN;
        }
        return ad.wrapping_mul(b).wrapping_add(a2);
    }
    let (a, b, c) = (a as u64, b as u64, c as u64);
    let mut a0 = a & 0xFFFF_FFFF;
    let mut a1 = a >> 32;
    let b0 = b & 0xFFFF_FFFF;
    let b1 = b >> 32;
    let mut t1 = a0.wrapping_mul(b1).wrapping_add(a1.wrapping_mul(b0));
    let t1a = t1 << 32;
    a0 = a0.wrapping_mul(b0).wrapping_add(t1a);
    a1 = a1
        .wrapping_mul(b1)
        .wrapping_add(t1 >> 32)
        .wrapping_add(u64::from(a0 < t1a));
    a0 = a0.wrapping_add(r as u64);
    a1 = a1.wrapping_add(u64::from(a0 < r as u64));
    for i in (0..64).rev() {
        a1 = a1.wrapping_add(a1.wrapping_add((a0 >> i) & 1));
        t1 = t1.wrapping_add(t1);
        if c <= a1 {
            a1 -= c;
            t1 += 1;
        }
    }
    if t1 > i64::MAX as u64 {
        return i64::MIN;
    }
    t1 as i64
}

/// Port of `int64_t av_rescale(int64_t a, int64_t b, int64_t c)` (`mathematics.c:134`).
pub fn av_rescale(a: i64, b: i64, c: i64) -> i64 {
    av_rescale_rnd(a, b, c, AV_ROUND_NEAR_INF)
}

/// Port of `int64_t av_compare_mod(uint64_t a, uint64_t b, uint64_t mod)` (`mathematics.c:160`).
pub fn av_compare_mod(a: u64, b: u64, r#mod: u64) -> i64 {
    let c = a.wrapping_sub(b) & r#mod.wrapping_sub(1);
    if c > (r#mod >> 1) {
        return c.wrapping_sub(r#mod) as i64;
    }
    c as i64
}

#[expect(
    clippy::excessive_precision,
    reason = "the literals of mathematics.c, verbatim, so they round to the same doubles"
)]
const P1: [f64; 15] = [
    -2.2335582639474375249e+15,
    -5.5050369673018427753e+14,
    -3.2940087627407749166e+13,
    -8.4925101247114157499e+11,
    -1.1912746104985237192e+10,
    -1.0313066708737980747e+08,
    -5.9545626019847898221e+05,
    -2.4125195876041896775e+03,
    -7.0935347449210549190e+00,
    -1.5453977791786851041e-02,
    -2.5172644670688975051e-05,
    -3.0517226450451067446e-08,
    -2.6843448573468483278e-11,
    -1.5982226675653184646e-14,
    -5.2487866627945699800e-18,
];
#[expect(
    clippy::excessive_precision,
    reason = "the literals of mathematics.c, verbatim, so they round to the same doubles"
)]
const Q1: [f64; 6] = [
    -2.2335582639474375245e+15,
    7.8858692566751002988e+12,
    -1.2207067397808979846e+10,
    1.0377081058062166144e+07,
    -4.8527560179962773045e+03,
    1.0,
];
#[expect(
    clippy::excessive_precision,
    reason = "the literals of mathematics.c, verbatim, so they round to the same doubles"
)]
const P2: [f64; 7] = [
    -2.2210262233306573296e-04,
    1.3067392038106924055e-02,
    -4.4700805721174453923e-01,
    5.5674518371240761397e+00,
    -2.3517945679239481621e+01,
    3.1611322818701131207e+01,
    -9.6090021968656180000e+00,
];
#[expect(
    clippy::excessive_precision,
    reason = "the literals of mathematics.c, verbatim, so they round to the same doubles"
)]
const Q2: [f64; 8] = [
    -5.5194330231005480228e-04,
    3.2547697594819615062e-02,
    -1.1151759188741312645e+00,
    1.3982595353892851542e+01,
    -6.0228002066743340583e+01,
    8.5539563258012929600e+01,
    -3.1446690275135491500e+01,
    1.0,
];

/// `eval_poly(p, y) / eval_poly(q, y)`, both polynomials stepped in one loop.
#[inline(always)]
fn ratio<const N: usize, const M: usize>(p: &[f64; N], q: &[f64; M], x: f64) -> f64 {
    let (mut a, mut b) = (0.0, 0.0);
    for i in (0..N.max(M)).rev() {
        if i + 1 == N {
            a = p[i];
        } else if i < N {
            a = a * x + p[i];
        }
        if i + 1 == M {
            b = q[i];
        } else if i < M {
            b = b * x + q[i];
        }
    }
    a / b
}

/// Port of `double av_bessel_i0(double x)` (`mathematics.c:257`): the modified Bessel function
/// of the first kind of order zero, by Blair and Edwards' rational approximations.
pub fn av_bessel_i0(x: f64) -> f64 {
    if x == 0.0 {
        return 1.0;
    }
    let x = x.abs();
    if x <= 15.0 {
        let y = x * x;
        return ratio(&P1, &Q1, y);
    }
    let y = 1.0 / x - 1.0 / 15.0;
    let r = ratio(&P2, &Q2, y);
    let factor = x.exp() / x.sqrt();
    factor * r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_examples() {
        assert_eq!(av_gcd(12, -18), 6);
        assert_eq!(av_rescale_rnd(3, 1, 2, 3 | AV_ROUND_PASS_MINMAX), 2);
        assert_eq!(
            av_rescale_rnd(i64::MIN, 1, 2, 3 | AV_ROUND_PASS_MINMAX),
            i64::MIN
        );
    }
}
