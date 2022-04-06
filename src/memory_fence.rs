#[cfg(target_arch = "xtensa")]
pub(crate) fn memory_fence() {
    unsafe {
        core::arch::asm!("memw");
    }
}

#[cfg(target_arch = "riscv32")]
pub(crate) fn memory_fence() {
    // no-op
}
