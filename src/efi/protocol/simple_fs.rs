use crate::efi::{Char16, EfiGuid, EfiStatus};
use alloc::vec::Vec;
use core::ops::AddAssign;
use core::ptr::null_mut;
pub const SIMPLE_FILE_SYSTEM_GUID: EfiGuid = EfiGuid::new(
    0x964E5B22,
    0x6459,
    0x11D2,
    [0x8E, 0x39, 0x0, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
);

pub const FILE_INFO_GUID: EfiGuid = EfiGuid::new(
    0x09576E92,
    0x6D3F,
    0x11D2,
    [0x8E, 0x39, 0x0, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
);

#[repr(C)]
pub struct SimpleFileSystem {
    pub revision: u64,

    open_volume:
        unsafe extern "efiapi" fn(this: *mut SimpleFileSystem, *mut *mut EfiFile) -> EfiStatus,
}

#[allow(unsafe_code)]
impl SimpleFileSystem {
    pub fn open_volume(&mut self) -> Result<*mut EfiFile, EfiStatus> {
        let mut file = null_mut();

        unsafe {
            let status = (self.open_volume)(self, &mut file);

            if status == 0 {
                Ok(file)
            } else {
                Err(status)
            }
        }
    }
}

#[repr(C)]
pub struct EfiFile {
    pub revision: u64,

    open: unsafe extern "efiapi" fn(
        this: *const EfiFile,
        file: *mut *mut EfiFile,
        file_name: *mut Char16,
        open_mode: u64,
        attributes: u64,
    ) -> EfiStatus,
    close: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    delete: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    read: unsafe extern "efiapi" fn(
        this: *const EfiFile,
        buffer_size: *mut u64,
        buffer: *mut u8,
    ) -> EfiStatus,
    write: unsafe extern "efiapi" fn(
        this: *const EfiFile,
        buffer_size: *mut u64,
        buffer: *const u8,
    ) -> EfiStatus,
    get_position: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    set_position: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    get_info: unsafe extern "efiapi" fn(
        this: *const EfiFile,
        *const EfiGuid,
        buffer_size: *mut usize,
        buffer: *mut u8,
    ) -> EfiStatus,
    set_info: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    flush: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    open_ex: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    read_ex: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    write_ex: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
    flush_ex: unsafe extern "efiapi" fn(this: *const EfiFile) -> EfiStatus,
}

#[allow(unsafe_code)]
impl EfiFile {
    pub fn open(
        &self,
        file_name: &str,
        open_mode: u64,
        attributes: u64,
    ) -> Result<*mut EfiFile, EfiStatus> {
        let mut file = null_mut();

        let mut vec = Vec::with_capacity(file_name.len());
        for c in file_name.encode_utf16() {
            vec.push(c);
        }

        unsafe {
            let status = (self.open)(self, file, vec.as_mut_ptr(), open_mode, attributes);

            if status == 0 {
                Ok(*file)
            } else {
                Err(status)
            }
        }
    }

    pub fn file_size(&self) -> u64 {
        let mut data = Vec::with_capacity(500);
        let mut buffer_size = data.capacity();

        unsafe {
            (self.get_info)(self, &FILE_INFO_GUID, &mut buffer_size, data.as_mut_ptr());
        }

        unsafe { *(data.as_ptr() as *const u64).offset(1) }
    }

    pub fn read(&self, buffer_size: *mut u64, buffer: *mut u8) -> EfiStatus {
        unsafe { (self.read)(self, buffer_size, buffer) }
    }

    pub fn read_chunked(&self, chunk_size: usize, buffer: &mut [u8]) -> EfiStatus {
        let len = buffer.len() - 1;
        let iter = len / chunk_size;
        let mut buf_size = chunk_size as u64;
        let mut remaining = len;
        let mut buffer_ptr = buffer.as_mut_ptr();

        for i in 0..iter {
            if remaining < chunk_size {
                buf_size = remaining as u64;
            }

            let status = unsafe { self.read(&mut buf_size, buffer_ptr) };
            unsafe { buffer_ptr = buffer_ptr.add(chunk_size) }

            if status != 0 {
                remaining -= chunk_size;
                return status;
            }
        }

        0
    }

    pub fn write_all(&self, buffer_size: *mut u64, buffer: *const u8) -> EfiStatus {
        unsafe { (self.write)(self, buffer_size, buffer) }
    }

    pub fn flush(&self) -> EfiStatus {
        unsafe { (self.flush)(self) }
    }

    pub fn close(&self) -> EfiStatus {
        unsafe { (self.close)(self) }
    }
}
