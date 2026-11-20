// A Rust port of FFmpeg n7.1 libavutil/adler32.c, itself a modified version of adler32.c from
// zlib. This is an altered version: it is not the original software.
//
// Copyright (C) 1995 Mark Adler
//
// This software is provided 'as-is', without any express or implied warranty. In no event will
// the authors be held liable for any damages arising from the use of this software.
//
// Permission is granted to anyone to use this software for any purpose, including commercial
// applications, and to alter it and redistribute it freely, subject to the following
// restrictions:
//
// 1. The origin of this software must not be misrepresented; you must not claim that you wrote
//    the original software. If you use this software in a product, an acknowledgment in the
//    product documentation would be appreciated but is not required.
// 2. Altered source versions must be plainly marked as such, and must not be misrepresented as
//    being the original software.
// 3. This notice may not be removed or altered from any source distribution.

//! The Rust port of `adler32.c`: safe Rust; `lib.rs` exports each function under its C name.

#![forbid(unsafe_code)]

const BASE: u64 = 65521;

/// Port of `AVAdler av_adler32_update(AVAdler adler, const uint8_t *buf, size_t len)` (`adler32.c:44`).
///
/// The 64-bit path (`HAVE_FAST_64BIT && HAVE_FAST_UNALIGNED && !CONFIG_SMALL`): eight
/// interleaved sums per 64-bit word, folded back into `s1` and `s2`.
pub fn av_adler32_update(adler: u32, buf: &[u8]) -> u32 {
    let mut s1 = u64::from(adler & 0xffff);
    let mut s2 = u64::from(adler >> 16);
    let mut rest = buf;
    while !rest.is_empty() {
        let len2 = ((rest.len() - 1) & !7).min(23 * 8);
        if len2 > 0 {
            let (mut a1, mut a2, mut b1, mut b2) = (0u64, 0u64, 0u64, 0u64);
            s2 = s2.wrapping_add(s1.wrapping_mul(len2 as u64));
            let (words, tail) = rest.split_at(len2);
            for word in words.as_chunks::<8>().0 {
                let v = u64::from_ne_bytes(*word);
                a2 = a2.wrapping_add(a1);
                b2 = b2.wrapping_add(b1);
                a1 = a1.wrapping_add(v & 0x00FF_00FF_00FF_00FF);
                b1 = b1.wrapping_add((v >> 8) & 0x00FF_00FF_00FF_00FF);
            }
            rest = tail;
            s1 = s1.wrapping_add(a1.wrapping_add(b1).wrapping_mul(0x1_0001_0001_0001) >> 48);
            let m = 0xFFFF_0000_FFFF;
            let folded = (a2 & m)
                .wrapping_add(b2 & m)
                .wrapping_add((a2 >> 16) & m)
                .wrapping_add((b2 >> 16) & m)
                .wrapping_mul(0x8_0000_0008)
                >> 32;
            s2 = s2.wrapping_add(folded).wrapping_add(lanes(a1, b1));
        }
        let Some((&byte, tail)) = rest.split_first() else {
            break;
        };
        s1 = s1.wrapping_add(u64::from(byte));
        s2 = s2.wrapping_add(s1);
        rest = tail;
        s1 %= BASE;
        s2 %= BASE;
    }
    ((s2 << 16) | s1) as u32
}

#[cfg(target_endian = "little")]
fn lanes(a1: u64, b1: u64) -> u64 {
    (2 * (a1.wrapping_mul(0x4_0003_0002_0001) >> 48))
        .wrapping_add(b1.wrapping_mul(0x1_0001_0001_0001) >> 48)
        .wrapping_add(2 * (b1.wrapping_mul(0x3_0002_0001_0000) >> 48))
}

#[cfg(target_endian = "big")]
fn lanes(a1: u64, b1: u64) -> u64 {
    (2 * (b1.wrapping_mul(0x1_0002_0003_0004) >> 48))
        .wrapping_add(a1.wrapping_mul(0x1_0001_0001_0001) >> 48)
        .wrapping_add(2 * (a1.wrapping_mul(0x0_0001_0002_0003) >> 48))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fate_vector() {
        let data: Vec<u8> = (0..7001u32)
            .map(|i| (((i * i) >> 3) + 123 * i) as u8)
            .collect();
        assert_eq!(av_adler32_update(1, &data), 0x50e6_e508);
    }

    #[test]
    fn empty_keeps_the_state() {
        assert_eq!(av_adler32_update(0xffff_ffff, &[]), 0xffff_ffff);
    }
}
