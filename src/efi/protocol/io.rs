use crate::efi::{Char16, EfiEvent, EfiStatus};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::fmt::Write;

#[repr(C)]
pub struct EfiInputKey {
    scan_code: u16,
    unicode_char: Char16,
}

#[repr(C)]
pub struct SimpleTextInputProtocol {
    reset: unsafe extern "efiapi" fn(
        this: *const SimpleTextInputProtocol,
        extended_verify: bool,
    ) -> EfiStatus,
    read_key: unsafe extern "efiapi" fn(
        this: *const SimpleTextInputProtocol,
        *mut EfiInputKey,
    ) -> EfiStatus,

    wait_for_key: EfiEvent,
}

#[repr(C)]
pub struct EfiSimpleTextOutputMode {
    max_mode: i32,

    mode: i32,
    attribute: i32,
    cursor_column: i32,
    cursor_row: i32,
    cursor_visible: bool,
}

#[repr(C)]
pub struct SimpleTextOutputProtocol {
    reset: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        extended_verify: bool,
    ) -> EfiStatus,

    output_string: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        string: *const Char16,
    ) -> EfiStatus,
    test_string: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        string: *const Char16,
    ) -> EfiStatus,

    query_mode: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        mode_number: u64,
        columns: *mut u64,
        rows: *mut u64,
    ) -> EfiStatus,
    set_mode: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        mode_number: u64,
    ) -> EfiStatus,
    set_attribute: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        attribute: u64,
    ) -> EfiStatus,

    clear_screen: unsafe extern "efiapi" fn(this: *const SimpleTextOutputProtocol) -> EfiStatus,
    set_cursor_position: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        column: u64,
        row: u64,
    ) -> EfiStatus,
    enable_cursor: unsafe extern "efiapi" fn(
        this: *const SimpleTextOutputProtocol,
        visible: bool,
    ) -> EfiStatus,

    mode: *const EfiSimpleTextOutputMode,
}

#[allow(unsafe_code)]
impl Write for SimpleTextOutputProtocol {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let string = String::from(s) + "\r\0";
        let bytes: Vec<u16> = string.encode_utf16().collect();

        unsafe {
            (self.output_string)(self, bytes.as_ptr());
        }

        Ok(())
    }
}

#[repr(C)]
pub struct DevicePathProtocol {
    device_type: u8,
    sub_type: u8,
    length: [u8; 2],
}
