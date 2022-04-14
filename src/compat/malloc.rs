use crate::memory_fence::memory_fence;
use log::trace;

extern "C" {
    static _heap_start: u8;
}

#[derive(Debug, Copy, Clone)]
struct Allocation {
    address: *const u8,
    size: usize,
    free: bool,
}

static mut ALLOCATIONS: [Option<Allocation>; 128] = [None; 128];
static mut ALLOC_INDEX: isize = -1;

pub unsafe extern "C" fn malloc(size: u32) -> *const u8 {
    trace!("malloc called {}", size);

    let mut candidate_addr = &_heap_start as *const u8;

    critical_section::with(|_critical_section| {
        let aligned_size = size + if size % 8 != 0 { 8 - size % 8 } else { 0 };

        // try to find a previously freed block
        let mut reused = 0 as *const u8;
        for allocation in ALLOCATIONS.iter_mut() {
            memory_fence();
            match allocation {
                Some(ref mut allocation) => {
                    if allocation.free && aligned_size <= allocation.size as u32 {
                        allocation.free = false;
                        reused = allocation.address;
                        break;
                    }
                }
                None => {}
            }
        }

        if reused.is_null() {
            // otherwise allocate after the highest allocated block
            if ALLOC_INDEX != -1 {
                candidate_addr = ALLOCATIONS[ALLOC_INDEX as usize]
                    .unwrap()
                    .address
                    .offset(ALLOCATIONS[ALLOC_INDEX as usize].unwrap().size as isize);
            }

            ALLOC_INDEX += 1;

            ALLOCATIONS[ALLOC_INDEX as usize] = Some(Allocation {
                address: candidate_addr,
                size: aligned_size as usize,
                free: false,
            });
            trace!("new allocation idx = {}", ALLOC_INDEX);
        } else {
            trace!("new allocation at reused block");
            candidate_addr = reused;
        }

        trace!("malloc at {:p}", candidate_addr);
    });

    return candidate_addr;
}

pub unsafe extern "C" fn free(ptr: *const u8) {
    trace!("free called {:p}", ptr);

    if ptr.is_null() {
        return;
    }

    critical_section::with(|_critical_section| {
        memory_fence();

        let alloced_idx = ALLOCATIONS.iter().enumerate().find(|(_, allocation)| {
            memory_fence();
            let addr = allocation.unwrap().address;
            allocation.is_some() && addr == ptr
        });

        if alloced_idx.is_some() {
            let alloced_idx = alloced_idx.unwrap().0;
            trace!("free idx {}", alloced_idx);

            if alloced_idx as isize == ALLOC_INDEX {
                ALLOCATIONS[alloced_idx] = None;
                ALLOC_INDEX -= 1;
            } else {
                ALLOCATIONS[alloced_idx] = ALLOCATIONS[alloced_idx as usize]
                    .take()
                    .and_then(|v| Some(Allocation { free: true, ..v }));
            }
        } else {
            panic!("freeing a memory area we don't know of. {:?}", ALLOCATIONS);
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn calloc(number: u32, size: u32) -> *const u8 {
    trace!("calloc {} {}", number, size);
    let ptr = malloc(number * size);
    let mut zp = ptr as *mut u8;
    for _ in 0..(number * size) {
        zp.write_volatile(0x00);
        zp = zp.offset(1);
    }
    ptr as *const u8
}
