use crate::efi::io::DevicePathProtocol;
use crate::efi::{
    Char16, EfiAllocateType, EfiGuid, EfiHandle, EfiMemoryType, EfiStatus, EfiTpl, PhysicalAddress,
    TableHeader,
};
use alloc::vec::Vec;
use core::ffi::c_void;
use core::fmt::{Display, Formatter, Pointer};
use core::ptr::{null, null_mut};
use core::slice;

#[repr(C)]
pub struct BootServices {
    header: TableHeader,

    raise_tpl: unsafe extern "efiapi" fn(new_tpl: EfiTpl) -> EfiTpl,
    restore_tpl: unsafe extern "efiapi" fn(old_tpl: EfiTpl),

    allocate_pages: unsafe extern "efiapi" fn(
        alloc_type: EfiAllocateType,
        memory_type: EfiMemoryType,
        pages: u64,
        memory: *mut PhysicalAddress,
    ) -> EfiStatus,
    free_pages: unsafe extern "efiapi" fn(memory: PhysicalAddress, pages: u64) -> EfiStatus,
    get_memory_map: unsafe extern "efiapi" fn(
        map_size: *mut u64,
        memory_map: *mut EfiMemoryDescriptor,
        map_key: *mut u64,
        descriptor_size: *mut u64,
        descriptor_version: *mut u32,
    ) -> EfiStatus,
    allocate_pool: unsafe extern "efiapi" fn(
        pool_type: EfiMemoryType,
        size: u64,
        buffer: *mut *mut u8,
    ) -> EfiStatus,
    free_pool: unsafe extern "efiapi" fn(buffer: *mut u8) -> EfiStatus,

    create_event: unsafe extern "efiapi" fn() -> EfiStatus,
    set_timer: unsafe extern "efiapi" fn() -> EfiStatus,
    wait_for_event: unsafe extern "efiapi" fn() -> EfiStatus,
    signal_event: unsafe extern "efiapi" fn() -> EfiStatus,
    close_event: unsafe extern "efiapi" fn() -> EfiStatus,
    check_event: unsafe extern "efiapi" fn() -> EfiStatus,

    install_protocol_interface: unsafe extern "efiapi" fn() -> EfiStatus,
    reinstall_protocol_interface: unsafe extern "efiapi" fn() -> EfiStatus,
    uninstall_protocol_interface: unsafe extern "efiapi" fn() -> EfiStatus,
    handle_protocol: unsafe extern "efiapi" fn() -> EfiStatus,
    reserved: *const c_void,
    register_protocol_notify: unsafe extern "efiapi" fn() -> EfiStatus,
    locate_handle: unsafe extern "efiapi" fn(
        search_type: SearchType,
        protocol: *const EfiGuid,
        key: *const u8,
        buffer_size: u64,
        buffer: *mut u8,
    ) -> EfiStatus,
    locate_device_path: unsafe extern "efiapi" fn() -> EfiStatus,
    install_configuration_table: unsafe extern "efiapi" fn() -> EfiStatus,

    load_image: unsafe extern "efiapi" fn(
        boot_policy: bool,
        parent_handle: EfiHandle,
        device_path: *const DevicePathProtocol,
        src_buffer: *const c_void,
        src_size: u64,
        handle: *mut EfiHandle,
    ) -> EfiStatus,
    start_image: unsafe extern "efiapi" fn(
        handle: EfiHandle,
        exit_data_size: *mut u64,
        exit_data: *mut *mut Char16,
    ) -> EfiStatus,
    exit: unsafe extern "efiapi" fn() -> EfiStatus,
    unload_image: unsafe extern "efiapi" fn() -> EfiStatus,
    exit_boot_services: unsafe extern "efiapi" fn(handle: EfiHandle, map_key: u64) -> EfiStatus,

    get_next_monotonic_count: unsafe extern "efiapi" fn() -> EfiStatus,
    stall: unsafe extern "efiapi" fn() -> EfiStatus,
    set_watchdog_timer: unsafe extern "efiapi" fn(
        timeout: u64,
        watchdog_code: u64,
        data_size: u64,
        watchdog_data: *mut Char16,
    ) -> EfiStatus,

    connect_controller: unsafe extern "efiapi" fn() -> EfiStatus,
    disconnect_controller: unsafe extern "efiapi" fn() -> EfiStatus,

    open_protocol: unsafe extern "efiapi" fn(
        efi_handle: EfiHandle,
        guid: EfiGuid,
        interface: *mut *mut c_void,
        agent: EfiHandle,
        controller: EfiHandle,
        attributes: u32,
    ) -> EfiStatus,
    CloseProtocol: unsafe extern "efiapi" fn() -> EfiStatus,
    OpenProtocolInformation: unsafe extern "efiapi" fn() -> EfiStatus,

    ProtocolsPerHandle: unsafe extern "efiapi" fn() -> EfiStatus,
    locate_handle_buffer: unsafe extern "efiapi" fn(
        search_type: SearchType,
        protocol: *const EfiGuid,
        key: *const u8,
        buffer_size: *mut u64,
        buffer: *mut *mut EfiHandle,
    ) -> EfiStatus,
    LocateProtocol: unsafe extern "efiapi" fn() -> EfiStatus,
    InstallMultipleProtocolInterface: unsafe extern "efiapi" fn() -> EfiStatus,
    UninstallMultipleProtocolInterface: unsafe extern "efiapi" fn() -> EfiStatus,

    CalculateCrc32: unsafe extern "efiapi" fn() -> EfiStatus,

    CopyMem: unsafe extern "efiapi" fn() -> EfiStatus,
    SetMem: unsafe extern "efiapi" fn() -> EfiStatus,
    CreateEventEx: unsafe extern "efiapi" fn() -> EfiStatus,
}

#[allow(unsafe_code)]
impl BootServices {
    pub fn allocate_pages(
        &self,
        alloc_type: EfiAllocateType,
        memory_type: EfiMemoryType,
        pages: u64,
        memory: *mut PhysicalAddress,
    ) -> EfiStatus {
        unsafe { (self.allocate_pages)(alloc_type, memory_type, pages, memory) }
    }

    pub fn allocate_pool(
        &self,
        pool_type: EfiMemoryType,
        size: u64,
        buffer: *mut *mut u8,
    ) -> EfiStatus {
        unsafe { (self.allocate_pool)(pool_type, size, buffer) }
    }

    pub fn free_pool(&self, buffer: *mut u8) -> EfiStatus {
        unsafe { (self.free_pool)(buffer) }
    }

    pub fn get_memory_map(
        &self,
        map_size: *mut u64,
        map: *mut EfiMemoryDescriptor,
        map_key: *mut u64,
        descriptor_size: *mut u64,
        descriptor_version: *mut u32,
    ) -> EfiStatus {
        unsafe {
            (self.get_memory_map)(map_size, map, map_key, descriptor_size, descriptor_version)
        }
    }

    pub fn locate_handle_for_protocol(
        &self,
        protocol: *const EfiGuid,
    ) -> Result<EfiHandle, EfiStatus> {
        let mut buffer = null_mut();
        let mut len = 0;

        unsafe {
            let status = (self.locate_handle_buffer)(
                SearchType::BY_PROTOCOL,
                protocol,
                null(),
                &mut len,
                &mut buffer,
            );

            if status != 0 {
                return Err(status);
            }
        }
        let slice = unsafe { slice::from_raw_parts(buffer, len as usize) };
        let first: EfiHandle = slice[0].clone();

        //Dealloc buffer
        unsafe { Vec::from_raw_parts(buffer, len as usize, len as usize) };

        Ok(first)
    }

    pub fn open_protocol<P>(
        &self,
        handle: EfiHandle,
        guid: EfiGuid,
        agent: EfiHandle,
    ) -> Result<*mut P, EfiStatus> {
        let mut ptr = null_mut();

        let status = unsafe {
            (self.open_protocol)(handle, guid, &mut ptr, agent, null_mut(), 0x20u32)
            //Attribute = Exclusive
        };

        if status != 0 {
            return Err(status);
        }

        Ok(ptr as *mut P)
    }

    pub fn exit_boot_services(&self, handle: EfiHandle, map_key: u64) -> EfiStatus {
        let func = self.exit_boot_services;

        unsafe { func(handle, map_key) }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct EfiMemoryDescriptor {
    pub memory_type: EfiMemoryType,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub num_pages: u64,
    attribute: u64,
}

impl Display for EfiMemoryDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Phys:{:X} | Virt: {:X} | {:?} | {}",
            self.physical_start,
            self.virtual_start,
            self.num_pages,
            match self.memory_type {
                EfiMemoryType::EFI_RESERVED_MEMORY_TYPE => {
                    "EfiReservedMemoryType"
                }
                EfiMemoryType::EFI_LOADER_CODE => {
                    "EfiLoaderCode"
                }
                EfiMemoryType::EFI_LOADER_DATA => {
                    "EfiLoaderData"
                }
                EfiMemoryType::EFI_BOOT_SERVICES_CODE => {
                    "EfiBootServicesCode"
                }
                EfiMemoryType::EFI_BOOT_SERVICES_DATA => {
                    "EfiBootServicesData"
                }
                EfiMemoryType::EFI_RUNTIME_SERVICES_CODE => {
                    "EfiRuntimeServicesCode"
                }
                EfiMemoryType::EFI_RUNTIME_SERVICES_DATA => {
                    "EfiRuntimeServicesData"
                }
                EfiMemoryType::EFI_CONVENTIONAL_MEMORY => {
                    "EfiConventionalMemory"
                }
                EfiMemoryType::EFI_UNUSABLE_MEMORY => {
                    "EfiUnusableMemory"
                }
                EfiMemoryType::EFI_ACPIRECLAIM_MEMORY => {
                    "EfiAcpiReclaimMemory"
                }
                EfiMemoryType::EFI_ACPIMEMORY_NVS => {
                    "EfiAcpiMemoryNVS"
                }
                EfiMemoryType::EFI_MEMORY_MAPPED_IO => {
                    "EfiMemoryMappedIO"
                }
                EfiMemoryType::EFI_MEMORY_MAPPED_IOPORT_SPACE => {
                    "EfiMemoryMappedIOPortSpace"
                }
                EfiMemoryType::EFI_PAL_CODE => {
                    "EfiPalCode"
                }
                EfiMemoryType::EFI_PERSISTENT_MEMORY => {
                    "EfiPersistentMemory"
                }
                EfiMemoryType::EFI_UNNACCEPTED_MEMORY_TYPE => {
                    "EfiUnacceptedMemoryType"
                }
                EfiMemoryType::EFI_MAX_MEMORY_TYPE => {
                    "EfiMaxMemoryType"
                }
                _ => {
                    "Undefined"
                }
            }
        )
    }
}

impl Default for EfiMemoryDescriptor {
    fn default() -> Self {
        EfiMemoryDescriptor {
            memory_type: EfiMemoryType::EFI_LOADER_CODE,
            physical_start: 0,
            virtual_start: 0,
            num_pages: 0,
            attribute: 0,
        }
    }
}

#[repr(transparent)]
pub struct SearchType(pub u32);

impl SearchType {
    pub const ALL_HANDLES: SearchType = SearchType(0);
    pub const BY_REGISTER_NOTIFY: SearchType = SearchType(1);
    pub const BY_PROTOCOL: SearchType = SearchType(2);
}
