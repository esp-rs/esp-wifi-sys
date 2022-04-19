use core::alloc::{GlobalAlloc, Layout};

use crate::compat::malloc::{free, malloc};

/// A simple allocator just using the internal `malloc` implementation.
/// Please note: This currently doesn't honor a non-standard aligment and will silently just use the default.
pub struct EspAllocator;

unsafe impl GlobalAlloc for EspAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // we don't care about the alignment here
        malloc(layout.size() as u32) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut u8);
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("Allocator error {:?}", layout);
}

#[global_allocator]
static GLOBAL: EspAllocator = EspAllocator;
