// A Rust port of FFmpeg n7.1 libavutil/avstring.c, changed from the C original.
// Copyright (c) 2000, 2001, 2002 Fabrice Bellard
// Copyright (c) 2007 Mans Rullgard
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of the string helpers of `avstring.c`; `lib.rs` exports each function under
//! its C name. `av_strireplace` (returns allocated memory) and `av_basename` (returns a static
//! string for an empty path) stay in C.

#![forbid(unsafe_code)]

use std::ffi::{c_char, c_int};

use crate::CChars;

/// The byte at `i` of a C string, `0` at its terminator (and past it, where C would read on).
fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// `av_tolower`: ASCII only.
fn lower(c: u8) -> u8 {
    c.to_ascii_lowercase()
}

/// Port of `char *av_stristr(const char *s1, const char *s2)` (`avstring.c:58`).
pub fn av_stristr(s1: Option<CChars<'_>>, s2: Option<CChars<'_>>) -> Option<usize> {
    let (mut s1, mut needle) = (s1?, s2?);
    let Some(first) = needle.next() else {
        return Some(0);
    };
    let mut i = 0;
    loop {
        let c = s1.next()?;
        if c.eq_ignore_ascii_case(&first)
            && needle
                .clone()
                .zip(s1.clone().chain(std::iter::repeat(0)))
                .all(|(p, c)| p.eq_ignore_ascii_case(&c))
        {
            return Some(i);
        }
        i += 1;
    }
}

/// Port of `char *av_strnstr(const char *haystack, const char *needle, size_t hay_length)` (`avstring.c:71`).
pub fn av_strnstr(
    haystack: Option<&[u8]>,
    needle: Option<&[u8]>,
    hay_length: usize,
) -> Option<usize> {
    let (haystack, needle) = (haystack?, needle?);
    if needle.is_empty() {
        return Some(0);
    }
    let windows = hay_length.checked_sub(needle.len())?.saturating_add(1);
    (0..windows)
        .take_while(|&start| start <= haystack.len())
        .find(|&start| haystack.get(start..start + needle.len()) == Some(needle))
}

/// Port of `size_t av_strlcpy(char *dst, const char *src, size_t size)` (`avstring.c:85`).
pub fn av_strlcpy(dst: &mut [u8], src: Option<&[u8]>) -> usize {
    let src = src.unwrap_or_default();
    let size = dst.len();
    let copied = src.len().min(size.saturating_sub(1));
    dst[..copied].copy_from_slice(&src[..copied]);
    if size > 0 {
        dst[copied] = 0;
    }
    src.len()
}

/// Port of `size_t av_strlcat(char *dst, const char *src, size_t size)` (`avstring.c:95`).
pub fn av_strlcat(dst: &mut [u8], src: Option<&[u8]>) -> usize {
    let len = dst.iter().position(|&b| b == 0).unwrap_or(dst.len());
    let src_len = src.map_or(0, <[u8]>::len);
    if dst.len() <= len + 1 {
        return len + src_len;
    }
    len + av_strlcpy(&mut dst[len..], src)
}

/// Port of `int av_strcasecmp(const char *a, const char *b)` (`avstring.c:207`).
pub fn av_strcasecmp(a: Option<CChars<'_>>, b: Option<CChars<'_>>) -> c_int {
    let (mut a, mut b) = match (a, b) {
        (Some(a), Some(b)) => (a, b),
        (a, b) => return ncasecmp(a.into_iter().flatten(), b.into_iter().flatten(), usize::MAX),
    };
    loop {
        let (c1, c2) = (lower(a.peek()), lower(b.peek()));
        if c1 == 0 || c1 != c2 {
            return c_int::from(c1) - c_int::from(c2);
        }
        a.next();
        b.next();
    }
}

/// Port of `int av_strncasecmp(const char *a, const char *b, size_t n)` (`avstring.c:217`).
pub fn av_strncasecmp(a: Option<CChars<'_>>, b: Option<CChars<'_>>, n: usize) -> c_int {
    ncasecmp(a.into_iter().flatten(), b.into_iter().flatten(), n)
}

/// `av_strncasecmp` over any bytes, `0` once a string ends.
fn ncasecmp(mut a: impl Iterator<Item = u8>, mut b: impl Iterator<Item = u8>, n: usize) -> c_int {
    if n == 0 {
        return 0;
    }
    let mut i = 0;
    loop {
        let (c1, c2) = (lower(a.next().unwrap_or(0)), lower(b.next().unwrap_or(0)));
        i += 1;
        if i == n || c1 == 0 || c1 != c2 {
            return c_int::from(c1) - c_int::from(c2);
        }
    }
}

/// `strncmp("ALL", s, n)` is zero.
fn is_all(s: &[u8], n: usize) -> bool {
    (0..n.min(4)).all(|i| at(b"ALL", i) == at(s, i))
}

/// Port of `int av_match_name(const char *name, const char *names)` (`avstring.c:345`).
pub fn av_match_name(name: Option<&[u8]>, names: Option<&[u8]>) -> c_int {
    let (Some(name), Some(mut names)) = (name, names) else {
        return 0;
    };
    while let Some(&first) = names.first() {
        let negate = first == b'-';
        let end = names.iter().position(|&b| b == b',').unwrap_or(names.len());
        let rest = &names[usize::from(negate)..];
        let seg = end - usize::from(negate);
        let len = seg.max(name.len());
        if ncasecmp(name.iter().copied(), rest.iter().copied(), len) == 0
            || is_all(rest, seg.max(3))
        {
            return c_int::from(!negate);
        }
        names = &names[(end + 1).min(names.len())..];
    }
    0
}

/// A C `char` as the platform's C does arithmetic on it (signed on Apple and x86).
fn signed(b: u8) -> c_int {
    c_int::from(b as c_char)
}

/// Port of `int av_match_list(const char *name, const char *list, char separator)` (`avstring.c:444`).
pub fn av_match_list(name: Option<&[u8]>, list: Option<&[u8]>, separator: c_char) -> c_int {
    let (Some(name), Some(list)) = (name, list) else {
        return 0;
    };
    let sep = separator as u8;
    let next = |s: &[u8], from: usize| -> Option<usize> {
        s[from..]
            .iter()
            .position(|&b| b == sep)
            .map(|i| from + i + 1)
    };
    let mut p = Some(0);
    while let Some(pi) = p.filter(|&pi| at(name, pi) != 0) {
        let mut q = Some(0);
        while let Some(qi) = q.filter(|&qi| at(list, qi) != 0) {
            let mut k = 0;
            loop {
                let (a, b) = (at(name, pi + k), at(list, qi + k));
                let joined = signed(a) * signed(b) == 0 && signed(a) + signed(b) == signed(sep);
                if !(a == b || joined) {
                    break;
                }
                if k > 0 && (a == 0 || a == sep) {
                    return 1;
                }
                k += 1;
            }
            q = next(list, qi);
        }
        p = next(name, pi);
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_and_appends() {
        let mut buf = [0xffu8; 6];
        assert_eq!(av_strlcpy(&mut buf, Some(b"abcdefgh")), 8);
        assert_eq!(&buf, b"abcde\0");
        let mut buf = *b"ab\0\0\0\0";
        assert_eq!(av_strlcat(&mut buf, Some(b"cdefg")), 7);
        assert_eq!(&buf, b"abcde\0");
    }

    #[test]
    fn matches_names() {
        assert_eq!(av_match_name(Some(b"mp4"), Some(b"mov,mp4,m4a")), 1);
        assert_eq!(av_match_name(Some(b"mp4"), Some(b"-mp4,ALL")), 0);
        assert_eq!(av_match_name(Some(b"x"), Some(b"ALL")), 1);
        assert_eq!(
            av_stristr(
                Some(CChars::new(c"Hello World")),
                Some(CChars::new(c"WORLD"))
            ),
            Some(6)
        );
        assert_eq!(av_strnstr(Some(b"abcdef"), Some(b"cd"), 4), Some(2));
        assert_eq!(av_strnstr(Some(b"abcdef"), Some(b"cd"), 3), None);
    }
}
