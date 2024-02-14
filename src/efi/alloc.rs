use crate::efi::{EfiMemoryType, SystemTable};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

pub struct EfiAllocator(pub *const SystemTable);

impl EfiAllocator {
    pub const fn new(system_table: *const SystemTable) -> EfiAllocator {
        EfiAllocator(system_table)
    }
}

#[allow(unsafe_code)]
unsafe impl Sync for EfiAllocator {}

#[allow(unsafe_code)]
unsafe impl GlobalAlloc for EfiAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let st = self.0.as_ref();

        if st.is_none() {
            return null_mut();
        }

        let mut ptr = null_mut();
        st.unwrap().boot_services().allocate_pool(
            EfiMemoryType::EFI_LOADER_DATA,
            layout.size() as u64,
            &mut ptr,
        );

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let st = self.0.as_ref();

        if st.is_none() {
            //Ignore any dealloc attempts as we are now in kernel and the kernel is responsible for managing the memory.
            return;
        }

        st.unwrap().boot_services().free_pool(ptr);
    }
}
