//! Convenience functions for interacting with ffi calls to vulkan.
//!
//! Here be dragons. Read the comments on functions to use them correctly!

use std::{convert::TryInto, ffi::CString, os::raw::c_char};

/// Build a vector of pointers to c-style strings from a vector of rust strings.
///
/// Unsafe because the returned vector of pointers is only valid while the
/// cstrings are alive.
pub unsafe fn to_os_ptrs(
    strings: &Vec<String>,
) -> (Vec<CString>, Vec<*const c_char>) {
    let cstrings = strings
        .iter()
        .cloned()
        .map(|str| CString::new(str).unwrap())
        .collect::<Vec<CString>>();
    let ptrs = cstrings
        .iter()
        .map(|cstr| cstr.as_ptr())
        .collect::<Vec<*const c_char>>();
    (cstrings, ptrs)
}

/// Copy a byte slice into a properly-aligned u32 array.
///
/// This is meant to help functions which use `include_bytes!` to load sprv
/// because Vulkan expects sprv source to be in u32 words but `include_bytes`
/// imports only u8 bytes.
///
/// A full copy is leveraged to handle endianess issues and to ensure proper
/// alignment.
///
/// Assumes that data is little endian and will break on other architectures.
///
pub fn copy_to_u32(bytes: &'static [u8]) -> Vec<u32> {
    const U32_SIZE: usize = std::mem::size_of::<u32>();
    if bytes.len() % U32_SIZE != 0 {
        panic!("the byte array must be evenly divisible into u32 words");
    }

    let mut buffer: Vec<u32> = vec![];
    let mut input: &[u8] = &bytes;
    while input.len() > 0 {
        let (int_slice, rest) = input.split_at(U32_SIZE);
        input = rest;
        let word = u32::from_le_bytes(int_slice.try_into().unwrap());
        buffer.push(word);
    }

    buffer
}
