use crate::efi::io::DevicePathProtocol;
use crate::efi::{EfiGuid, EfiHandle, SystemTable};

pub const LOADED_IMAGE_GUID: EfiGuid = EfiGuid::new(
    0x5B1B31A1,
    0x9562,
    0x11D2,
    [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
);

#[repr(C)]
pub struct LoadedImage {
    revision: u32,

    parent_handle: EfiHandle,

    system_table: *const SystemTable,

    pub device_handle: EfiHandle,
    pub file_path: *const DevicePathProtocol,
}
