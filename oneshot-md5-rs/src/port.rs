// A Rust port of FFmpeg n7.1 libavutil/md5.c, changed from the C original.
// Copyright (C) 2006 Michael Niedermayer
// Copyright (C) 2003-2005 by Christopher R. Hertel
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of `md5.c` (context, update, final), exported as `oneshot_md5`: the one-shot
//! wrapper around `av_md5_init`, `av_md5_update` and `av_md5_final` that the recordings call.

#![forbid(unsafe_code)]

use std::ffi::c_int;

const S: [[u32; 4]; 4] = [
    [7, 12, 17, 22],
    [5, 9, 14, 20],
    [4, 11, 16, 23],
    [6, 10, 15, 21],
];

/// `T` of `md5.c`: `T[i] = floor(abs(sin(i + 1)) * 2^32)`.
const T: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

/// `AVMD5`: the byte count, the pending block, and the state (stored D, C, B, A as in C).
struct Md5 {
    len: u64,
    block: [u8; 64],
    abcd: [u32; 4],
}

/// One step of `CORE` for step `$i`, with the round's function picked at compile time.
macro_rules! core {
    ($x:ident, $i:expr, $a:ident, $b:ident, $c:ident, $d:ident) => {{
        const I: usize = $i;
        let (f, k) = match I {
            0..16 => ($d ^ ($b & ($c ^ $d)), I & 15),
            16..32 => (($d & $b) | (!$d & $c), (1 + 5 * I) & 15),
            32..48 => ($b ^ $c ^ $d, (5 + 3 * I) & 15),
            _ => ($c ^ ($b | !$d), (7 * I) & 15),
        };
        $a = $a.wrapping_add(T[I]).wrapping_add(f).wrapping_add($x[k]);
        $a = $b.wrapping_add($a.rotate_left(S[I >> 4][I & 3]));
    }};
}

/// `CORE4` of `md5.c`: sixteen steps, unrolled as the C is.
macro_rules! core4 {
    ($x:ident, $i:expr, $a:ident, $b:ident, $c:ident, $d:ident) => {
        core!($x, $i, $a, $b, $c, $d);
        core!($x, $i + 1, $d, $a, $b, $c);
        core!($x, $i + 2, $c, $d, $a, $b);
        core!($x, $i + 3, $b, $c, $d, $a);
        core!($x, $i + 4, $a, $b, $c, $d);
        core!($x, $i + 5, $d, $a, $b, $c);
        core!($x, $i + 6, $c, $d, $a, $b);
        core!($x, $i + 7, $b, $c, $d, $a);
        core!($x, $i + 8, $a, $b, $c, $d);
        core!($x, $i + 9, $d, $a, $b, $c);
        core!($x, $i + 10, $c, $d, $a, $b);
        core!($x, $i + 11, $b, $c, $d, $a);
        core!($x, $i + 12, $a, $b, $c, $d);
        core!($x, $i + 13, $d, $a, $b, $c);
        core!($x, $i + 14, $c, $d, $a, $b);
        core!($x, $i + 15, $b, $c, $d, $a);
    };
}

fn body(abcd: &mut [u32; 4], block: &[u8; 64]) {
    let mut x = [0u32; 16];
    for (word, bytes) in x.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_le_bytes(*bytes);
    }
    let (mut a, mut b, mut c, mut d) = (abcd[3], abcd[2], abcd[1], abcd[0]);
    core4!(x, 0, a, b, c, d);
    core4!(x, 16, a, b, c, d);
    core4!(x, 32, a, b, c, d);
    core4!(x, 48, a, b, c, d);
    abcd[0] = abcd[0].wrapping_add(d);
    abcd[1] = abcd[1].wrapping_add(c);
    abcd[2] = abcd[2].wrapping_add(b);
    abcd[3] = abcd[3].wrapping_add(a);
}

impl Md5 {
    fn new() -> Self {
        Self {
            len: 0,
            block: [0; 64],
            abcd: [0x1032_5476, 0x98ba_dcfe, 0xefcd_ab89, 0x6745_2301],
        }
    }

    fn update(&mut self, mut src: &[u8]) {
        let j = (self.len & 63) as usize;
        self.len = self.len.wrapping_add(src.len() as u64);
        if j > 0 {
            let cnt = src.len().min(64 - j);
            self.block[j..j + cnt].copy_from_slice(&src[..cnt]);
            src = &src[cnt..];
            if j + cnt < 64 {
                return;
            }
            let block = self.block;
            body(&mut self.abcd, &block);
        }
        let (blocks, rest) = src.as_chunks::<64>();
        for block in blocks {
            body(&mut self.abcd, block);
        }
        self.block[..rest.len()].copy_from_slice(rest);
    }

    fn finish(mut self) -> [u8; 16] {
        let count = (self.len << 3).to_le_bytes();
        self.update(&[0x80]);
        while self.len & 63 != 56 {
            self.update(&[0]);
        }
        self.update(&count);
        let mut out = [0u8; 16];
        for (i, word) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            word.copy_from_slice(&self.abcd[3 - i].to_le_bytes());
        }
        out
    }
}

/// Port of `int oneshot_md5(uint8_t *dst, int dst_size, const uint8_t *src, size_t len, int chunk)` (`oneshot_md5.c:10`):
/// the MD5 of `src`, fed to the context `chunk` bytes at a time (all at once when `chunk` <= 0).
pub fn oneshot_md5(dst: &mut [u8], src: &[u8], chunk: c_int) -> c_int {
    if dst.len() < 16 {
        return -1;
    }
    let mut ctx = Md5::new();
    match usize::try_from(chunk) {
        Ok(n) if n > 0 => src.chunks(n).for_each(|piece| ctx.update(piece)),
        _ => ctx.update(src),
    }
    dst[..16].copy_from_slice(&ctx.finish());
    16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_1321_vectors() {
        let mut out = [0u8; 16];
        assert_eq!(oneshot_md5(&mut out, b"abc", 0), 16);
        assert_eq!(
            out,
            [
                0x90, 0x01, 0x50, 0x98, 0x3c, 0xd2, 0x4f, 0xb0, 0xd6, 0x96, 0x3f, 0x7d, 0x28, 0xe1,
                0x7f, 0x72
            ]
        );
        oneshot_md5(&mut out, b"", 0);
        assert_eq!(out[..4], [0xd4, 0x1d, 0x8c, 0xd9]);
    }
}
