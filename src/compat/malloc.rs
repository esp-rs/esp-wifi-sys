use core::alloc::{GlobalAlloc, Layout};

use crate::ALLOCATOR;

pub unsafe extern "C" fn malloc(size: u32) -> *const u8 {
    // FIXME: what is the correct value for `alignment`?
    ALLOCATOR.alloc(Layout::from_size_align_unchecked(size as usize, 1))
}

pub unsafe extern "C" fn free(ptr: *const u8) {
    ALLOCATOR.dealloc(ptr as *mut u8, Layout::for_value_raw(ptr));
}

#[no_mangle]
pub unsafe extern "C" fn calloc(number: u32, size: u32) -> *const u8 {
    // FIXME: what is the correct value for `alignment`?
    let mut ptr = ALLOCATOR.alloc(Layout::from_size_align_unchecked(size as usize, 1)) as *mut u8;

    for _ in 0..(number * size) {
        ptr.write_volatile(0xFF);
        ptr = ptr.offset(1);
    }

    ptr as *const u8
}
