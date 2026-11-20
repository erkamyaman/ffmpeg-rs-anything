//! C ABI for `rational.c`: drop-in replacements with the original names and signatures, generated
//! by rust-anything from `rust-anything.json`. All of the crate's `unsafe` code lives in this file.

pub mod port;

mod __rust_anything_c_abi {

    /// A pointer to one value as a reference; `None` for `NULL`.
    pub(super) fn out<'a, T>(ptr: *mut T) -> Option<&'a mut T> {
        // SAFETY: the C API passes NULL or a valid, aligned pointer used by nothing else
        // during the call.
        unsafe { ptr.as_mut() }
    }
}

/// `int av_reduce(int *dst_num, int *dst_den, int64_t num, int64_t den, int64_t max)` (`rational.c:35`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_reduce(
    dst_num: *mut std::ffi::c_int,
    dst_den: *mut std::ffi::c_int,
    num: i64,
    den: i64,
    max: i64,
) -> std::ffi::c_int {
    port::av_reduce(
        crate::__rust_anything_c_abi::out(dst_num),
        crate::__rust_anything_c_abi::out(dst_den),
        num,
        den,
        max,
    )
}

/// `av_reduce` as `rust_anything_av_reduce`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_reduce(
    dst_num: *mut std::ffi::c_int,
    dst_den: *mut std::ffi::c_int,
    num: i64,
    den: i64,
    max: i64,
) -> std::ffi::c_int {
    port::av_reduce(
        crate::__rust_anything_c_abi::out(dst_num),
        crate::__rust_anything_c_abi::out(dst_den),
        num,
        den,
        max,
    )
}
