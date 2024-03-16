#![no_std]
#![no_main]
#![deny(unsafe_code)]

extern crate alloc;

use crate::common::log::{FrameBuffer, FrameBufferInfo, Logger};
use crate::common::paging::{PageMapTable, PageMapTableBuilder};
use crate::efi::alloc::EfiAllocator;
use crate::efi::graphics::{GraphicsOutput, GRAPHICS_OUTPUT_GUID};
use crate::efi::loaded_image::{LoadedImage, LOADED_IMAGE_GUID};
use crate::efi::logger::EfiLogger;
use crate::efi::simple_fs::{EfiFile, SimpleFileSystem, SIMPLE_FILE_SYSTEM_GUID};
use crate::efi::SystemTable;
use crate::efi::{format_efi_status, EfiHandle, EfiMemoryDescriptor, EfiStatus};
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::arch::asm;
use core::fmt::{Debug, Write};
use core::mem::{align_of, size_of};
use core::panic::PanicInfo;
use core::ptr::{null, null_mut};
use core::{mem, slice};
use elf_loader::{ElfFile, ProgramHeader};

mod common;
mod efi;

#[global_allocator]
static mut ALLOC: EfiAllocator = EfiAllocator::new(null());

static mut LOGGER: Option<*mut EfiLogger> = None;

#[allow(unsafe_code)]
fn logger() -> &'static mut EfiLogger {
    unsafe { &mut *LOGGER.unwrap() }
}

#[allow(unsafe_code)]
#[export_name = "efi_main"]
fn main(handle: EfiHandle, system_table: *const SystemTable) -> u64 {
    //Set allocator
    unsafe {
        ALLOC = EfiAllocator::new(system_table);
    }

    let st = SystemTable::from_ptr(system_table);

    //Get the handles for the Graphics Output Protocol for the logger
    let graphics_handle_result = st
        .boot_services()
        .locate_handle_for_protocol(&GRAPHICS_OUTPUT_GUID);
    if graphics_handle_result.is_err() {
        let _ = writeln!(
            st.stdout(),
            "Unable to locate graphics handle: {}",
            format_efi_status(graphics_handle_result.unwrap_err())
        );
        panic!();
    }

    let graphics_handle = graphics_handle_result.unwrap();

    let g_result = st.boot_services().open_protocol::<GraphicsOutput>(
        graphics_handle,
        GRAPHICS_OUTPUT_GUID,
        handle,
    );
    if g_result.is_err() {
        let _ = writeln!(
            st.stdout(),
            "Unable to load graphics protocol: {}",
            format_efi_status(g_result.unwrap_err())
        );
        panic!();
    }

    let g = unsafe { &*g_result.unwrap() };
    let mode = g.mode();
    let mode_info = mode.current_mode();

    //Save the framebuffer data so we can use it in kernel later and access logger
    let framebuffer = FrameBufferInfo {
        address: mode.framebuffer as usize,
        len: mode.framebuffer_len as usize,
        screen_width: mode_info.horizontal_resolution,
        screen_height: mode_info.vertical_resolution,
        pixels_per_scan_line: mode_info.pixels_per_scan_line,
    };

    let mut fb = FrameBuffer::new(framebuffer.clone());
    let mut logger = EfiLogger::new(fb.clone());

    unsafe {
        LOGGER = Some(&mut logger);
    }

    writeln!(
        logger,
        "EfiLogger initialized, using framebuffer at {:X}\r",
        framebuffer.address
    )
    .unwrap();
    //Load kernel data from /kernel.elf
    let kernel_data = load_kernel(handle, st).expect("Unable to load kernel");
    let kernel_file = ElfFile::from(kernel_data);
    //let program_headers = kernel_file.program_headers();

    if !kernel_file.verify_magic() {
        panic!(
            "Can not load kernel: File is corrupted, it is not of type ELF\r\nKernel Length: {:?}",
            kernel_file.len()
        )
    }

    //Load the main function into variable
    let kernel_main: unsafe extern "C" fn(*const KernelArgs) -> u64 =
        unsafe { mem::transmute(kernel_file.entry_point_ptr() as *const (*const KernelArgs)) };

    kernel_file
        .program_headers()
        .iter()
        .filter(|h| h.header_type == 0x1)
        .for_each(|h| {
            let offset = h.offset;
            let v_addr = h.v_addr;
            let mem_size = h.memory_size;
            writeln!(
                logger,
                "Abs: {:#016X} - V: {:#016X} - {:8} Bytes\r",
                kernel_file.data() as u64 + offset,
                v_addr,
                mem_size
            )
            .unwrap();
        });

    let headers = kernel_file.program_headers();

    //Prepare page table
    //let page_table = init_page_table(&kernel_file);

    //Retrieve RSDP
    //TODO: Read RSDP

    let mut kargs = KernelArgs {
        rsd_ptr: &0u8,
        memory_map: &0u8,
        memory_map_size: 0,
        memory_map_type: 0,
        framebuffer,
    };

    //We need the memory map for our kernel
    let memory_map = exit_boot_services(handle, st).unwrap();

    kargs.memory_map = memory_map.as_ptr() as *const u8;
    kargs.memory_map_size = memory_map.len() as u64;
    kargs.memory_map_type = MemoryMapType::UEFI;

    writeln!(
        logger,
        "Memory map is at: {:X}\r",
        kargs.memory_map as usize
    )
    .expect("");

    // for i in 0..memory_map.len() {
    //     let entry = memory_map.get(i);
    //
    //     if entry.is_some() {
    //         let entry = entry.unwrap();
    //
    //         writeln!(logger, "{}\r", entry).unwrap();
    //     }
    // }

    writeln!(
        logger,
        "KArgs address: {:X}\r",
        &kargs as *const KernelArgs as u64
    )
    .unwrap();

    //Write the kernel to 0x0 in physical memory
    allocate_kernel(&kernel_file, &headers);

    //Set page table
    // unsafe {
    //     asm!("mov rax, {}",
    //         "mov cr3, rax",
    //         in(reg) page_table.as_ptr());
    // }

    unsafe {
        //logger.log("Passing control to kernel");
        let returnval = kernel_main(&kargs as *const KernelArgs);
        writeln!(logger, "Kernel returned: {:X}\r", returnval).unwrap();
    }
    loop {}
}

#[allow(unsafe_code)]
pub fn load_kernel(handle: EfiHandle, st: &SystemTable) -> Result<Vec<u8>, EfiStatus> {
    let mut logger = unsafe { &mut *LOGGER.unwrap() };
    //logger.log("NightOS Bootloader (v0.0.1)\r\n");

    //logger.log("Loading kernel from /kernel.elf\r\n");
    let loaded_image_result =
        st.boot_services()
            .open_protocol::<LoadedImage>(handle, LOADED_IMAGE_GUID, handle);
    if loaded_image_result.is_err() {
        panic!(
            "Unable to open LoadedImage protocol: {}",
            loaded_image_result.unwrap_err()
        );
    }
    let loaded_image = loaded_image_result.unwrap();

    let sfp_result = unsafe {
        st.boot_services().open_protocol::<SimpleFileSystem>(
            (*loaded_image).device_handle,
            SIMPLE_FILE_SYSTEM_GUID,
            (*loaded_image).device_handle,
        )
    };
    if sfp_result.is_err() {
        panic!(
            "Unable to open SimpleFileSystem protocol: {}",
            sfp_result.unwrap_err()
        );
    }
    let sfp = sfp_result.unwrap();

    let root = unsafe {
        (*sfp)
            .open_volume()
            .expect("Unable to open volume due to error")
    };

    let file = unsafe {
        (*root)
            .open("kernel.elf", 1, 0)
            .expect("Unable to open file")
    };
    let file_size = unsafe { (*file).file_size() };

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(file_size as usize, 0);
    let mut buffer_size = buffer.capacity();

    unsafe {
        let status = (*file).read_chunked(256, &mut buffer);

        if status == 0 {
            Ok(buffer)
        } else {
            Err(status)
        }
    }
}

#[allow(unsafe_code)]
pub fn allocate_kernel(file: &ElfFile, headers: &[ProgramHeader]) {
    let logger = logger();
    let file_data = unsafe { slice::from_raw_parts(file.data(), file.len()) };

    //Copy the kernel file to start of physical memory
    for header in headers {
        let start = header.v_addr as usize;

        let mut ptr = start as *mut u8;
        let start_file = header.offset as usize;
        let end_file = start_file + header.memory_size as usize;

        //writeln!(logger, "Copy file {:#016X} to memory {:#016X}\r", start_file, start).unwrap();

        for i in start_file..end_file {
            unsafe {
                *ptr = file_data[i];
                ptr = ptr.add(1);
            }
        }

        ptr = start as *mut u8;

        //unsafe { writeln!(logger, "File {:#016X} = Memory {:#016X}\r", file_data[start_file], *ptr).unwrap(); }
    }
}

#[allow(unsafe_code)]
pub fn exit_boot_services(
    handle: EfiHandle,
    st: &SystemTable,
) -> Result<Vec<EfiMemoryDescriptor>, EfiStatus> {
    let logger = logger();

    let mut bytes: Vec<u8> = Vec::new();
    let mut map_size_bytes = 0u64;
    let mut map_key = 0u64;
    let mut desc_size = 0u64;
    let mut desc_version = 0u32;

    //Query size
    let mut status = st.boot_services().get_memory_map(
        &mut map_size_bytes,
        bytes.as_mut_ptr() as *mut EfiMemoryDescriptor,
        &mut map_key,
        &mut desc_size,
        &mut desc_version,
    );
    map_size_bytes += 2 * desc_size;
    bytes.resize(map_size_bytes as usize, 0);

    let map_len = map_size_bytes / desc_size;
    let mut map = Vec::with_capacity(map_len as usize);

    status = st.boot_services().get_memory_map(
        &mut map_size_bytes,
        bytes.as_mut_ptr() as *mut EfiMemoryDescriptor,
        &mut map_key,
        &mut desc_size,
        &mut desc_version,
    );

    //Now we need to exit the UEFI boot services and call the kernel main to hand over control
    let exit_status = st.boot_services().exit_boot_services(handle, map_key);

    //Disable EFI Allocator as kernel will manage memory from now
    unsafe {
        ALLOC = EfiAllocator::new(null());
        writeln!(*LOGGER.unwrap(), "Exiting Boot Services: {}\r", exit_status).expect("");
    }

    //Add the descriptors to the correctly aligned map
    unsafe {
        for i in 0..map_len as usize {
            let pos = i * desc_size as usize;
            let pos_end = pos + desc_size as usize;
            let data = &bytes[pos..pos_end];

            let descriptor = data.as_ptr() as *const EfiMemoryDescriptor;
            let descriptor_ref = &*descriptor;

            //(*LOGGER.unwrap()).log(format!("D! {}\r\n", *descriptor).as_str());

            map.push(*descriptor_ref);
            //(*LOGGER.unwrap()).log(format!("D3! {}\r\n", map.get(i).unwrap()).as_str());
        }
    }

    return Ok(map);
}

#[allow(unsafe_code)]
pub fn init_page_table(kernel: &ElfFile) -> Vec<PageMapTable> {
    let mut page_maps: Vec<PageMapTable> = Vec::new();
    page_maps.resize(512 + 512 + 512 + 512, PageMapTable::from(0u64));
    let pdp_ptr = page_maps.get(512).unwrap() as *const _ as u64;
    let pd_ptr = page_maps.get(1024).unwrap() as *const _ as u64;
    let pt_ptr = page_maps.get(1536).unwrap() as *const _ as u64;

    //PML4
    page_maps.insert(
        0,
        PageMapTableBuilder::from(0u64)
            .address(pdp_ptr)
            .present(true)
            .into(),
    );

    //PDP
    page_maps.insert(
        512,
        PageMapTableBuilder::from(0u64)
            .address(pd_ptr)
            .present(true)
            .into(),
    );

    //Page Directory
    page_maps.insert(
        1024,
        PageMapTableBuilder::from(0u64)
            .address(pt_ptr)
            .present(true)
            .into(),
    );

    let loadable_headers: Vec<ProgramHeader> = kernel
        .program_headers()
        .iter()
        .filter(|hdr| hdr.header_type == 0x1 && hdr.align == 0x1000)
        .cloned()
        .collect();

    //Page Table
    //This only prepares a basic page table containing the kernel only
    for header in loadable_headers {
        let pages = if header.memory_size % 0x1000 == 0 {
            header.memory_size / 0x1000
        } else {
            header.memory_size / 0x1000 + 1
        };

        // Pages are 4 KB in size, so we need to convert v_addr into page index
        let page_start = header.v_addr / 4096;

        for page in 0..pages {
            page_maps.insert(
                (1536 + page_start + page) as usize,
                PageMapTableBuilder::from(0u64)
                    .address(header.p_addr + page * 0x1000)
                    .present(true)
                    .into(),
            );
        }
    }

    //Add the page table itself to the end of the page table
    for i in 0..4 {
        unsafe {
            page_maps.insert(
                2043 + i,
                PageMapTableBuilder::from(0u64)
                    .address(page_maps.as_ptr().add(i * 8 * 512) as u64)
                    .present(true)
                    .into(),
            );
        }
    }

    page_maps
}

#[allow(unsafe_code)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        if LOGGER.is_some() {
            (*LOGGER.unwrap()).log(info.to_string().as_str());
        }
    }
    loop {}
}

struct MemoryMapType;

impl MemoryMapType {
    pub const UEFI: u8 = 1;
}

#[repr(C)]
pub struct KernelArgs {
    rsd_ptr: *const u8,
    memory_map: *const u8,
    memory_map_size: u64,
    memory_map_type: u8,
    framebuffer: FrameBufferInfo,
}

#[repr(C)]
struct KernelElf {
    loadable_headers: Vec<ProgramHeader>,
    start_address: u64,
}
