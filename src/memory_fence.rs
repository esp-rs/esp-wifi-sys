pub(crate) fn memory_fence() {
    #[cfg(target_arch = "xtensa")]
    unsafe {
        core::arch::asm!("memw");
    }
}
