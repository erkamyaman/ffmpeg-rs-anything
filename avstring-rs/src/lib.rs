//! C ABI for `avstring.c`: drop-in replacements with the original names and signatures, generated
//! by rust-anything from `rust-anything.json`. All of the crate's `unsafe` code lives in this file.

pub mod port;

use std::ffi::c_char;

/// A `const char *` read one byte at a time up to its NUL, without measuring it first: for ports
/// where `strlen` would cost more than the work (`av_strcasecmp` on short words). A port opts in
/// by typing the parameter `Option<crate::CChars<'_>>`; `rust-anything refresh` then passes it.
#[derive(Clone, Debug)]
pub struct CChars<'a> {
    ptr: std::ptr::NonNull<std::ffi::c_char>,
    life: std::marker::PhantomData<&'a std::ffi::CStr>,
}

impl<'a> CChars<'a> {
    /// The bytes of `s`, for calling a port from Rust.
    pub fn new(s: &'a std::ffi::CStr) -> Self {
        Self {
            ptr: std::ptr::NonNull::from(s).cast(),
            life: std::marker::PhantomData,
        }
    }

    /// The next byte without moving past it; `0` at the end.
    pub fn peek(&self) -> u8 {
        // SAFETY: `ptr` points into a NUL-terminated string valid for `'a`, at or before its NUL.
        unsafe { *self.ptr.as_ptr() as u8 }
    }

    /// The rest of the string as a slice, without its NUL (this measures it).
    pub fn to_bytes(&self) -> &'a [u8] {
        // SAFETY: as in `peek`; the rest of the string is itself NUL-terminated.
        unsafe { std::ffi::CStr::from_ptr(self.ptr.as_ptr()) }.to_bytes()
    }
}

impl Iterator for CChars<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let b = self.peek();
        if b == 0 {
            return None;
        }
        // SAFETY: `b` is not the NUL, so the next byte is still inside the string.
        self.ptr = unsafe { self.ptr.add(1) };
        Some(b)
    }
}

impl std::iter::FusedIterator for CChars<'_> {}

mod __rust_anything_c_abi {
    /// A `const char *` argument read as the port goes; `None` for `NULL`.
    pub(super) fn c_chars<'a>(ptr: *const std::ffi::c_char) -> Option<super::CChars<'a>> {
        std::ptr::NonNull::new(ptr.cast_mut()).map(|ptr| super::CChars {
            ptr,
            life: std::marker::PhantomData,
        })
    }
    use std::ffi::{CStr, c_char};

    /// A `const char *` argument as bytes; `None` for `NULL`.
    pub(super) fn c_str<'a>(ptr: *const c_char) -> Option<&'a [u8]> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: the C API takes NUL-terminated strings that stay valid for the call;
        // NULL is handled above.
        Some(unsafe { CStr::from_ptr(ptr) }.to_bytes())
    }

    /// A writable byte pointer and its size as a slice; empty for `NULL` or a zero size.
    pub(super) fn bytes_mut<'a>(ptr: *mut u8, len: usize) -> &'a mut [u8] {
        if ptr.is_null() || len == 0 {
            return &mut [];
        }

        // SAFETY: the C API passes `len` writable bytes at `ptr`, used by nothing else during the
        // call; NULL is handled above.
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }

    /// A C length as `usize`; a negative length is zero.
    pub(super) fn len<T: TryInto<usize>>(n: T) -> usize {
        n.try_into().unwrap_or(0)
    }
}

/// `char *av_stristr(const char *s1, const char *s2)` (`avstring.c:58`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_stristr(s1: *const c_char, s2: *const c_char) -> *mut std::ffi::c_char {
    match port::av_stristr(
        crate::__rust_anything_c_abi::c_chars(s1),
        crate::__rust_anything_c_abi::c_chars(s2),
    ) {
        Some(offset) => s1.cast_mut().cast::<u8>().wrapping_add(offset).cast(),
        None => std::ptr::null_mut::<std::ffi::c_char>(),
    }
}

/// `av_stristr` as `rust_anything_av_stristr`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_stristr(
    s1: *const c_char,
    s2: *const c_char,
) -> *mut std::ffi::c_char {
    match port::av_stristr(
        crate::__rust_anything_c_abi::c_chars(s1),
        crate::__rust_anything_c_abi::c_chars(s2),
    ) {
        Some(offset) => s1.cast_mut().cast::<u8>().wrapping_add(offset).cast(),
        None => std::ptr::null_mut::<std::ffi::c_char>(),
    }
}

/// `char *av_strnstr(const char *haystack, const char *needle, size_t hay_length)` (`avstring.c:71`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_strnstr(
    haystack: *const c_char,
    needle: *const c_char,
    hay_length: usize,
) -> *mut std::ffi::c_char {
    match port::av_strnstr(
        crate::__rust_anything_c_abi::c_str(haystack),
        crate::__rust_anything_c_abi::c_str(needle),
        hay_length,
    ) {
        Some(offset) => haystack.cast_mut().cast::<u8>().wrapping_add(offset).cast(),
        None => std::ptr::null_mut::<std::ffi::c_char>(),
    }
}

/// `av_strnstr` as `rust_anything_av_strnstr`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_strnstr(
    haystack: *const c_char,
    needle: *const c_char,
    hay_length: usize,
) -> *mut std::ffi::c_char {
    match port::av_strnstr(
        crate::__rust_anything_c_abi::c_str(haystack),
        crate::__rust_anything_c_abi::c_str(needle),
        hay_length,
    ) {
        Some(offset) => haystack.cast_mut().cast::<u8>().wrapping_add(offset).cast(),
        None => std::ptr::null_mut::<std::ffi::c_char>(),
    }
}

/// `size_t av_strlcpy(char *dst, const char *src, size_t size)` (`avstring.c:85`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_strlcpy(dst: *mut std::ffi::c_char, src: *const c_char, size: usize) -> usize {
    port::av_strlcpy(
        crate::__rust_anything_c_abi::bytes_mut(
            dst.cast::<u8>(),
            crate::__rust_anything_c_abi::len(size),
        ),
        crate::__rust_anything_c_abi::c_str(src),
    )
}

/// `av_strlcpy` as `rust_anything_av_strlcpy`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_strlcpy(
    dst: *mut std::ffi::c_char,
    src: *const c_char,
    size: usize,
) -> usize {
    port::av_strlcpy(
        crate::__rust_anything_c_abi::bytes_mut(
            dst.cast::<u8>(),
            crate::__rust_anything_c_abi::len(size),
        ),
        crate::__rust_anything_c_abi::c_str(src),
    )
}

/// `size_t av_strlcat(char *dst, const char *src, size_t size)` (`avstring.c:95`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_strlcat(dst: *mut std::ffi::c_char, src: *const c_char, size: usize) -> usize {
    port::av_strlcat(
        crate::__rust_anything_c_abi::bytes_mut(
            dst.cast::<u8>(),
            crate::__rust_anything_c_abi::len(size),
        ),
        crate::__rust_anything_c_abi::c_str(src),
    )
}

/// `av_strlcat` as `rust_anything_av_strlcat`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_strlcat(
    dst: *mut std::ffi::c_char,
    src: *const c_char,
    size: usize,
) -> usize {
    port::av_strlcat(
        crate::__rust_anything_c_abi::bytes_mut(
            dst.cast::<u8>(),
            crate::__rust_anything_c_abi::len(size),
        ),
        crate::__rust_anything_c_abi::c_str(src),
    )
}

/// `int av_strcasecmp(const char *a, const char *b)` (`avstring.c:207`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_strcasecmp(a: *const c_char, b: *const c_char) -> std::ffi::c_int {
    port::av_strcasecmp(
        crate::__rust_anything_c_abi::c_chars(a),
        crate::__rust_anything_c_abi::c_chars(b),
    )
}

/// `av_strcasecmp` as `rust_anything_av_strcasecmp`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_strcasecmp(
    a: *const c_char,
    b: *const c_char,
) -> std::ffi::c_int {
    port::av_strcasecmp(
        crate::__rust_anything_c_abi::c_chars(a),
        crate::__rust_anything_c_abi::c_chars(b),
    )
}

/// `int av_strncasecmp(const char *a, const char *b, size_t n)` (`avstring.c:217`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_strncasecmp(a: *const c_char, b: *const c_char, n: usize) -> std::ffi::c_int {
    port::av_strncasecmp(
        crate::__rust_anything_c_abi::c_chars(a),
        crate::__rust_anything_c_abi::c_chars(b),
        n,
    )
}

/// `av_strncasecmp` as `rust_anything_av_strncasecmp`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_strncasecmp(
    a: *const c_char,
    b: *const c_char,
    n: usize,
) -> std::ffi::c_int {
    port::av_strncasecmp(
        crate::__rust_anything_c_abi::c_chars(a),
        crate::__rust_anything_c_abi::c_chars(b),
        n,
    )
}

/// `int av_match_name(const char *name, const char *names)` (`avstring.c:345`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_match_name(name: *const c_char, names: *const c_char) -> std::ffi::c_int {
    port::av_match_name(
        crate::__rust_anything_c_abi::c_str(name),
        crate::__rust_anything_c_abi::c_str(names),
    )
}

/// `av_match_name` as `rust_anything_av_match_name`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_match_name(
    name: *const c_char,
    names: *const c_char,
) -> std::ffi::c_int {
    port::av_match_name(
        crate::__rust_anything_c_abi::c_str(name),
        crate::__rust_anything_c_abi::c_str(names),
    )
}

/// `int av_match_list(const char *name, const char *list, char separator)` (`avstring.c:444`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_match_list(
    name: *const c_char,
    list: *const c_char,
    separator: std::ffi::c_char,
) -> std::ffi::c_int {
    port::av_match_list(
        crate::__rust_anything_c_abi::c_str(name),
        crate::__rust_anything_c_abi::c_str(list),
        separator,
    )
}

/// `av_match_list` as `rust_anything_av_match_list`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_match_list(
    name: *const c_char,
    list: *const c_char,
    separator: std::ffi::c_char,
) -> std::ffi::c_int {
    port::av_match_list(
        crate::__rust_anything_c_abi::c_str(name),
        crate::__rust_anything_c_abi::c_str(list),
        separator,
    )
}
