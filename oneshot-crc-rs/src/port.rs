// A Rust port of FFmpeg n7.1 libavutil/crc.c, changed from the C original.
// Copyright (c) 2006 Michael Niedermayer
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of `crc.c` (`av_crc_init`, `av_crc_get_table`, `av_crc`), exported as
//! `oneshot_crc` and `oneshot_crc_custom`: the one-shot wrappers the recordings call.

#![forbid(unsafe_code)]

use std::sync::OnceLock;

const EINVAL: i64 = -22;

/// `(le, bits, poly)` of each `AVCRCId`, in the enum's order.
const PARAMS: [(bool, u32, u32); 8] = [
    (false, 8, 0x07),
    (false, 16, 0x8005),
    (false, 16, 0x1021),
    (false, 32, 0x04C1_1DB7),
    (true, 32, 0xEDB8_8320),
    (true, 16, 0xA001),
    (false, 24, 0x0086_4CFB),
    (false, 8, 0x1D),
];

/// `av_crc_init`: a 257-entry table (byte at a time) or a 1024-entry one (four bytes at a time).
fn init(le: bool, bits: u32, poly: u32, size: usize) -> Vec<u32> {
    let mut ctx = vec![0u32; size];
    for i in 0..256u32 {
        let mut c;
        if le {
            c = i;
            for _ in 0..8 {
                c = (c >> 1) ^ (poly & (c & 1).wrapping_neg());
            }
            ctx[i as usize] = c;
        } else {
            c = i << 24;
            for _ in 0..8 {
                c = (c << 1) ^ ((poly << (32 - bits)) & (((c as i32) >> 31) as u32));
            }
            ctx[i as usize] = c.swap_bytes();
        }
    }
    ctx[256] = 1;
    if size >= 1024 {
        for j in 0..3 {
            for i in 0..256 {
                let prev = ctx[256 * j + i];
                ctx[256 * (j + 1) + i] = (prev >> 8) ^ ctx[(prev & 0xFF) as usize];
            }
        }
    }
    ctx
}

/// `av_crc`.
fn crc(ctx: &[u32], mut crc: u32, buffer: &[u8]) -> u32 {
    let mut rest = buffer;
    if ctx[256] == 0
        && let Some(t) = ctx.first_chunk::<1024>()
        && let [t0, t1, t2, t3] = t.as_chunks::<256>().0
    {
        let (words, tail) = buffer.as_chunks::<4>();
        for word in words {
            let [b0, b1, b2, b3] = (crc ^ u32::from_le_bytes(*word)).to_le_bytes();
            crc = t3[usize::from(b0)]
                ^ t2[usize::from(b1)]
                ^ t1[usize::from(b2)]
                ^ t0[usize::from(b3)];
        }
        rest = tail;
    }
    let Some(t) = ctx.first_chunk::<256>() else {
        return crc;
    };
    for &b in rest {
        crc = t[usize::from(crc as u8 ^ b)] ^ (crc >> 8);
    }
    crc
}

/// Port of `uint32_t oneshot_crc(int id, uint32_t crc, const uint8_t *buf, size_t len)` (`oneshot_crc.c:9`):
/// `av_crc(av_crc_get_table(id), crc, buf, len)`. C aborts on an unknown `id`; this returns 0.
pub fn oneshot_crc(id: i32, crc_in: u32, buf: &[u8]) -> u32 {
    static TABLES: [OnceLock<Vec<u32>>; 8] = [const { OnceLock::new() }; 8];
    let Some((slot, &(le, bits, poly))) = usize::try_from(id)
        .ok()
        .and_then(|i| Some((TABLES.get(i)?, PARAMS.get(i)?)))
    else {
        return 0;
    };
    crc(slot.get_or_init(|| init(le, bits, poly, 1024)), crc_in, buf)
}

/// Port of `int64_t oneshot_crc_custom(int le, int bits, uint32_t poly, int table_size, uint32_t crc, const uint8_t *buf, size_t len)` (`oneshot_crc.c:14`):
/// builds a table with `av_crc_init` and runs `av_crc`, or returns `av_crc_init`'s error.
pub fn oneshot_crc_custom(
    le: i32,
    bits: i32,
    poly: u32,
    table_size: i32,
    crc_in: u32,
    buf: &[u8],
) -> i64 {
    if !(8..=32).contains(&bits) || i64::from(poly) >= 1i64 << bits {
        return EINVAL;
    }
    let size = match table_size {
        257 => 257,
        1024 => 1024,
        _ => return EINVAL,
    };
    let table = init(le != 0, bits as u32, poly, size);
    i64::from(crc(&table, crc_in, buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fate_vectors() {
        let buf: Vec<u8> = (0..1999u32).map(|i| (i + i * i) as u8).collect();
        let expected = [
            (4, 0x3D5C_DD04),
            (3, 0xE0BA_F5C0),
            (6, 0x0032_6039),
            (5, 0xBFD8),
            (1, 0xBB1F),
            (0, 0xE3),
            (7, 0xD6),
        ];
        for (id, want) in expected {
            assert_eq!(oneshot_crc(id, 0, &buf), want, "id {id}");
        }
    }
}
