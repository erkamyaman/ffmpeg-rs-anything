// A Rust port of FFmpeg n7.1 libavutil/rational.c, changed from the C original.
// Copyright (c) 2003 Michael Niedermayer
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of `av_reduce` from `rational.c`; `lib.rs` exports it under its C name. The
//! functions taking `AVRational` by value stay in C (the bridge does not carry structs).

#![forbid(unsafe_code)]

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
}
