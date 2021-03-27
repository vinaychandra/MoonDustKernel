use x86_64::VirtAddr;

extern "C" {
    static mut __tdata_start: usize;
    static mut __tdata_end: usize;
    static mut __tbss_start: usize;
    static mut __tbss_end: usize;
}

/// Initialize CPU local store for kernel.
/// This can be called per-CPU for TLS data for the core.
pub fn initialize_tls() {
    let total_size;
    let tls_ptr = unsafe {
        let tdata_size =
            &__tdata_end as *const usize as usize - &__tdata_start as *const usize as usize;
        total_size = &__tbss_end as *const usize as usize - &__tdata_start as *const usize as usize;
        crate::common::memory::cpu_local::load_tls_data(
            &__tdata_start as *const usize as *const u8,
            tdata_size,
            total_size + 8, // Add 8 bytes to store TCB pointer.
        )
    };
    info!(target: "initialize_tls", "TLS data loaded. Setting fs");
    let fs_ptr = ((tls_ptr as *const u8 as u64) + (total_size as u64)) as *mut u64;
    x86_64::registers::model_specific::FsBase::write(VirtAddr::from_ptr(fs_ptr));
    unsafe {
        // SystemV Abi needs [fs:0] to be the value of fs
        *fs_ptr = fs_ptr as u64;
    }
    info!(target: "initialize_tls", "TLS Pointer is set to {:x?}. Size is {:?} bytes", fs_ptr, total_size);
}
