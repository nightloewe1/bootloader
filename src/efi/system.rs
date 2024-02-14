use crate::efi::io::SimpleTextOutputProtocol;
use crate::efi::{BootServices, Char16, EfiHandle, TableHeader};
use core::ffi::c_void;

#[repr(C)]
pub struct SystemTable {
    header: TableHeader,

    firmware_vendor: *const Char16,
    firmware_revision: u32,

    console_in_handle: EfiHandle,
    console_in: *const u64,

    console_out_handle: EfiHandle,
    console_out: *mut SimpleTextOutputProtocol,

    std_err_handle: EfiHandle,
    stderr: *const u64,

    runtime_services: *const c_void,
    boot_services: *const BootServices,

    table_size: u64,
    config_table: *const u64,
}

#[allow(unsafe_code)]
impl SystemTable {
    pub fn from_ptr(system_table: *const SystemTable) -> &'static SystemTable {
        unsafe { &*system_table }
    }

    pub fn boot_services(&self) -> &BootServices {
        unsafe { &*self.boot_services }
    }

    pub fn stdout(&self) -> &mut SimpleTextOutputProtocol {
        unsafe { &mut *self.console_out }
    }
}
