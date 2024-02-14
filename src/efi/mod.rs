pub mod alloc;
mod boot;
pub mod logger;
mod protocol;
mod runtime;
mod system;

pub use boot::*;
pub use protocol::*;
pub use system::*;

#[repr(C)]
struct TableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
}

pub type Char16 = u16;

pub type PhysicalAddress = u64;
pub type VirtualAddress = u64;

pub type EfiHandle = *mut core::ffi::c_void;
pub type EfiEvent = *mut core::ffi::c_void;

pub type EfiStatus = u64;
pub type EfiTpl = u64;

pub type EfiAllocateType = u32;
pub const ALLOCATE_ANY_PAGES: EfiAllocateType = 0;
pub const ALLOCATE_MAX_ADDRESS: EfiAllocateType = 1;
pub const ALLOCATE_ADDRESS: EfiAllocateType = 2;
pub const MAX_ALLOCATE_TYPE: EfiAllocateType = 3;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct EfiMemoryType(pub u32);

impl EfiMemoryType {
    pub const EFI_RESERVED_MEMORY_TYPE: EfiMemoryType = EfiMemoryType(0);
    pub const EFI_LOADER_CODE: EfiMemoryType = EfiMemoryType(1);
    pub const EFI_LOADER_DATA: EfiMemoryType = EfiMemoryType(2);
    pub const EFI_BOOT_SERVICES_CODE: EfiMemoryType = EfiMemoryType(3);
    pub const EFI_BOOT_SERVICES_DATA: EfiMemoryType = EfiMemoryType(4);
    pub const EFI_RUNTIME_SERVICES_CODE: EfiMemoryType = EfiMemoryType(5);
    pub const EFI_RUNTIME_SERVICES_DATA: EfiMemoryType = EfiMemoryType(6);
    pub const EFI_CONVENTIONAL_MEMORY: EfiMemoryType = EfiMemoryType(7);
    pub const EFI_UNUSABLE_MEMORY: EfiMemoryType = EfiMemoryType(8);
    pub const EFI_ACPIRECLAIM_MEMORY: EfiMemoryType = EfiMemoryType(9);
    pub const EFI_ACPIMEMORY_NVS: EfiMemoryType = EfiMemoryType(10);
    pub const EFI_MEMORY_MAPPED_IO: EfiMemoryType = EfiMemoryType(11);
    pub const EFI_MEMORY_MAPPED_IOPORT_SPACE: EfiMemoryType = EfiMemoryType(12);
    pub const EFI_PAL_CODE: EfiMemoryType = EfiMemoryType(13);
    pub const EFI_PERSISTENT_MEMORY: EfiMemoryType = EfiMemoryType(14);
    pub const EFI_UNNACCEPTED_MEMORY_TYPE: EfiMemoryType = EfiMemoryType(15);
    pub const EFI_MAX_MEMORY_TYPE: EfiMemoryType = EfiMemoryType(16);
}

#[repr(C)]
pub struct EfiGuid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

impl EfiGuid {
    pub const fn new(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> EfiGuid {
        EfiGuid {
            data1,
            data2,
            data3,
            data4,
        }
    }
}

pub fn format_efi_status(status: EfiStatus) -> u64 {
    if status & (1 << 63) == (1 << 63) {
        status & !(1 << 63)
    } else {
        status
    }
}
