// A Rust port of FFmpeg n7.1 libavutil/rational.c, changed from the C original.
// Copyright (c) 2003 Michael Niedermayer
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of `rational.c`; `lib.rs` exports each function under its C name.
//! `av_find_nearest_q_idx` (reads a list through a pointer) and `av_gcd_q` stay in C.

#![forbid(unsafe_code)]

use crate::AVRational;

const INT_MAX: i64 = i32::MAX as i64;
const AV_ROUND_DOWN: i32 = 2;
const AV_ROUND_UP: i32 = 3;
const AV_ROUND_NEAR_INF: i32 = 5;

/// `av_gcd` of `mathematics.c`.
fn gcd(a: i64, b: i64) -> i64 {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    let (za, zb) = (a.trailing_zeros(), b.trailing_zeros());
    let mut u = (a >> za).wrapping_abs();
    let mut v = (b >> zb).wrapping_abs();
    while u != v {
        if u > v {
            std::mem::swap(&mut u, &mut v);
        }
        v -= u;
        v >>= v.trailing_zeros();
    }
    ((u as u64) << za.min(zb)) as i64
}

/// `x * f + g` as C computes it with `x` a `uint64_t` and `f`, `g` ints.
fn lin(x: u64, f: i32, g: i32) -> u64 {
    x.wrapping_mul(f as i64 as u64)
        .wrapping_add(g as i64 as u64)
}

/// Port of `int av_reduce(int *dst_num, int *dst_den, int64_t num, int64_t den, int64_t max)` (`rational.c:35`):
/// the closest fraction to `num/den` with both terms at most `max`; 1 when exact.
pub fn av_reduce(
    dst_num: Option<&mut i32>,
    dst_den: Option<&mut i32>,
    num: i64,
    den: i64,
    max: i64,
) -> i32 {
    let (mut a0, mut a1) = ((0i32, 1i32), (1i32, 0i32));
    let sign = (num < 0) != (den < 0);
    let g = gcd(num.wrapping_abs(), den.wrapping_abs());
    let (mut num, mut den) = (num, den);
    if g != 0 {
        num = num.wrapping_abs().wrapping_div(g);
        den = den.wrapping_abs().wrapping_div(g);
    }
    if num <= max && den <= max {
        a1 = (num as i32, den as i32);
        den = 0;
    }
    while den != 0 {
        let mut x = num.wrapping_div(den) as u64;
        let next_den = (num as u64).wrapping_sub((den as u64).wrapping_mul(x)) as i64;
        let a2n = lin(x, a1.0, a0.0) as i64;
        let a2d = lin(x, a1.1, a0.1) as i64;
        if a2n > max || a2d > max {
            if a1.0 != 0 {
                x = max
                    .wrapping_sub(i64::from(a0.0))
                    .wrapping_div(i64::from(a1.0)) as u64;
            }
            if a1.1 != 0 {
                x = x.min(
                    max.wrapping_sub(i64::from(a0.1))
                        .wrapping_div(i64::from(a1.1)) as u64,
                );
            }
            let lhs = (den as u64).wrapping_mul(lin(x.wrapping_mul(2), a1.1, a0.1));
            let rhs = num.wrapping_mul(i64::from(a1.1)) as u64;
            if lhs > rhs {
                a1 = (lin(x, a1.0, a0.0) as i32, lin(x, a1.1, a0.1) as i32);
            }
            break;
        }
        a0 = a1;
        a1 = (a2n as i32, a2d as i32);
        num = den;
        den = next_den;
    }
    if let Some(n) = dst_num {
        *n = if sign { a1.0.wrapping_neg() } else { a1.0 };
    }
    if let Some(d) = dst_den {
        *d = a1.1;
    }
    i32::from(den == 0)
}

/// `av_rescale_rnd` of `mathematics.c` for the rounding modes this file uses (no
/// `AV_ROUND_PASS_MINMAX`).
fn rescale_rnd(a: i64, b: i64, c: i64, rnd: i32) -> i64 {
    if c <= 0 || b < 0 {
        return i64::MIN;
    }
    if a < 0 {
        let flipped = rnd ^ ((rnd >> 1) & 1);
        return (rescale_rnd(-a.max(-i64::MAX), b, c, flipped) as u64).wrapping_neg() as i64;
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

/// `av_cmp_q` of `rational.h`: -1, 0 or 1, `INT_MIN` when either is 0/0.
fn cmp_q(a: AVRational, b: AVRational) -> i32 {
    let tmp = i64::from(a.num)
        .wrapping_mul(i64::from(b.den))
        .wrapping_sub(i64::from(b.num) * i64::from(a.den));
    if tmp != 0 {
        (((tmp ^ i64::from(a.den) ^ i64::from(b.den)) >> 63) as i32) | 1
    } else if b.den != 0 && a.den != 0 {
        0
    } else if a.num != 0 && b.num != 0 {
        (a.num >> 31) - (b.num >> 31)
    } else {
        i32::MIN
    }
}

/// `av_reduce` into an `AVRational`, bounded by `INT_MAX` unless `max` says otherwise.
fn reduced(num: i64, den: i64, max: i64) -> AVRational {
    let (mut n, mut d) = (0, 0);
    av_reduce(Some(&mut n), Some(&mut d), num, den, max);
    AVRational { num: n, den: d }
}

/// Port of `AVRational av_mul_q(AVRational b, AVRational c)` (`rational.c:80`).
pub fn av_mul_q(b: AVRational, c: AVRational) -> AVRational {
    reduced(
        i64::from(b.num) * i64::from(c.num),
        i64::from(b.den) * i64::from(c.den),
        INT_MAX,
    )
}

/// Port of `AVRational av_div_q(AVRational b, AVRational c)` (`rational.c:88`).
pub fn av_div_q(b: AVRational, c: AVRational) -> AVRational {
    av_mul_q(
        b,
        AVRational {
            num: c.den,
            den: c.num,
        },
    )
}

/// Port of `AVRational av_add_q(AVRational b, AVRational c)` (`rational.c:93`).
pub fn av_add_q(b: AVRational, c: AVRational) -> AVRational {
    reduced(
        (i64::from(b.num) * i64::from(c.den)).wrapping_add(i64::from(c.num) * i64::from(b.den)),
        i64::from(b.den) * i64::from(c.den),
        INT_MAX,
    )
}

/// Port of `AVRational av_sub_q(AVRational b, AVRational c)` (`rational.c:101`).
pub fn av_sub_q(b: AVRational, c: AVRational) -> AVRational {
    av_add_q(
        b,
        AVRational {
            num: c.num.wrapping_neg(),
            den: c.den,
        },
    )
}

/// The exponent `frexp` gives `d` (finite): `d = m * 2^e` with `0.5 <= |m| < 1`, 0 for 0.
fn frexp_exponent(d: f64) -> i32 {
    if d == 0.0 {
        return 0;
    }
    let bits = d.abs().to_bits();
    let biased = (bits >> 52) as i32;
    if biased == 0 {
        return -1022 - (bits.leading_zeros() as i32 - 12);
    }
    biased - 1022
}

/// Port of `AVRational av_d2q(double d, int max)` (`rational.c:106`): the closest fraction to
/// `d` with terms at most `max`; 0/0 for NaN, ±1/0 beyond `INT_MAX`.
pub fn av_d2q(d: f64, max: i32) -> AVRational {
    if d.is_nan() {
        return AVRational { num: 0, den: 0 };
    }
    if d.abs() > (INT_MAX + 3) as f64 {
        return AVRational {
            num: if d < 0.0 { -1 } else { 1 },
            den: 0,
        };
    }
    let exponent = (frexp_exponent(d) - 1).max(0);
    let den = 1i64 << (62 - exponent);
    let num = (d * den as f64 + 0.5).floor() as i64;
    let a = reduced(num, den, i64::from(max));
    if (a.num == 0 || a.den == 0) && d != 0.0 && max > 0 && max < i32::MAX {
        return reduced(num, den, INT_MAX);
    }
    a
}

/// Port of `int av_nearer_q(AVRational q, AVRational q1, AVRational q2)` (`rational.c:127`): 1
/// if `q1` is nearer to `q` than `q2`, -1 if `q2` is, 0 if both are as near.
pub fn av_nearer_q(q: AVRational, q1: AVRational, q2: AVRational) -> i32 {
    let a =
        (i64::from(q1.num) * i64::from(q2.den)).wrapping_add(i64::from(q2.num) * i64::from(q1.den));
    let b = (2 * i64::from(q1.den)).wrapping_mul(i64::from(q2.den));
    let x_up = rescale_rnd(a, i64::from(q.den), b, AV_ROUND_UP);
    let x_down = rescale_rnd(a, i64::from(q.den), b, AV_ROUND_DOWN);
    let side = i32::from(x_up > i64::from(q.num)) - i32::from(x_down < i64::from(q.num));
    side.wrapping_mul(cmp_q(q2, q1))
}

/// `av_log2` of an `unsigned`: the index of the highest set bit, 0 for 0.
fn log2(v: i32) -> i32 {
    31 - (v as u32 | 1).leading_zeros() as i32
}

/// `av_rescale(num, 1 << shift, den)`, or `av_rescale(num, 1, den << -shift)` for a negative
/// `shift`.
fn scaled(num: i64, den: i64, shift: i32) -> i64 {
    if shift >= 0 {
        rescale_rnd(num, 1i64 << shift, den, AV_ROUND_NEAR_INF)
    } else {
        rescale_rnd(
            num,
            1,
            den.wrapping_shl(shift.unsigned_abs()),
            AV_ROUND_NEAR_INF,
        )
    }
}

/// Port of `uint32_t av_q2intfloat(AVRational q)` (`rational.c:152`): `q` as the bits of the
/// nearest IEEE single. Negating `INT_MIN` overflows in C; ffmpeg's build (clang -O3) tests the
/// sign of the wrapped `num`, divides by 2^31 and keeps x/0 positive, as here.
pub fn av_q2intfloat(q: AVRational) -> u32 {
    let mut num = q.num;
    if q.den < 0 {
        num = num.wrapping_neg();
    }
    let sign = i32::from(num < 0);
    let num = i64::from(num.unsigned_abs());
    let den = i64::from(q.den.unsigned_abs());
    if num == 0 && den == 0 {
        return 0xFFC0_0000;
    }
    if num == 0 {
        return 0;
    }
    if den == 0 {
        return 0x7F80_0000;
    }
    let mut shift = 23 + log2(den as i32) - log2(num as i32);
    let n = scaled(num, den, shift);
    shift -= i32::from(n >= 1 << 24);
    shift += i32::from(n < 1 << 23);
    let n = scaled(num, den, shift);
    let high = sign.wrapping_shl(31) | (150 - shift).wrapping_shl(23);
    (i64::from(high) | (n - (1 << 23))) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduces() {
        let (mut n, mut d) = (0, 0);
        assert_eq!(
            av_reduce(Some(&mut n), Some(&mut d), 30000, 1001 * 10, 1 << 30),
            1
        );
        assert_eq!((n, d), (3000, 1001));
        assert_eq!(
            av_reduce(Some(&mut n), Some(&mut d), 314_159_265, 100_000_000, 1000),
            0
        );
        assert_eq!((n, d), (355, 113));
    }

    #[test]
    fn arithmetic() {
        let q = |num, den| AVRational { num, den };
        assert_eq!(av_mul_q(q(1, 2), q(2, 3)), q(1, 3));
        assert_eq!(av_add_q(q(1, 2), q(1, 3)), q(5, 6));
        assert_eq!(av_sub_q(q(1, 2), q(1, 3)), q(1, 6));
        assert_eq!(av_div_q(q(1, 2), q(1, 4)), q(2, 1));
        assert_eq!(av_d2q(0.5, 255), q(1, 2));
        assert_eq!(av_d2q(f64::NAN, 255), q(0, 0));
        assert_eq!(av_q2intfloat(q(1, 2)), 0.5f32.to_bits());
        assert_eq!(av_nearer_q(q(1, 3), q(1, 2), q(1, 4)), -1);
    }
}
