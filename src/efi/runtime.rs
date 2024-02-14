use crate::efi::{Char16, EfiGuid, EfiStatus, TableHeader};
use core::ffi::CStr;

#[repr(C)]
pub struct RuntimeServices {
    header: TableHeader,

    get_time: unsafe extern "efiapi" fn() -> EfiStatus,
    set_time: unsafe extern "efiapi" fn() -> EfiStatus,
    get_wakeup_time: unsafe extern "efiapi" fn() -> EfiStatus,
    set_wakeup_time: unsafe extern "efiapi" fn() -> EfiStatus,

    set_virtual_address_map: unsafe extern "efiapi" fn() -> EfiStatus,
    convert_ptr: unsafe extern "efiapi" fn() -> EfiStatus,

    pub get_var: unsafe extern "efiapi" fn() -> EfiStatus,
    get_next_variable_name: unsafe extern "efiapi" fn() -> EfiStatus,
    pub set_var: unsafe extern "efiapi" fn(
        key: *mut Char16,
        vendor_guid: *const EfiGuid,
        attributes: u32,
        size: u64,
        data: *mut u8,
    ) -> EfiStatus,

    unused: unsafe extern "efiapi" fn() -> EfiStatus,
    reset_system: unsafe extern "efiapi" fn() -> EfiStatus,

    update_capsule: unsafe extern "efiapi" fn() -> EfiStatus,
    query_capsule_features: unsafe extern "efiapi" fn() -> EfiStatus,

    query_variable_info: unsafe extern "efiapi" fn() -> EfiStatus,
}

impl RuntimeServices {
    pub fn set_var(&self, key: &str, data: &mut u8) {}
}
