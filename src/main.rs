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
use crate::efi::simple_fs::{SimpleFileSystem, SIMPLE_FILE_SYSTEM_GUID};
use crate::efi::SystemTable;
use crate::efi::{format_efi_status, EfiHandle, EfiMemoryDescriptor, EfiStatus};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::arch::asm;
use core::fmt::{Debug, Write};
use core::ops::Add;
use core::panic::PanicInfo;
use core::ptr::null;
use core::{mem, slice};
use elf_loader::{ElfFile, ProgramHeader, RelocationSection};

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

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Step 1: Prepare logger                                                                     //
    ////////////////////////////////////////////////////////////////////////////////////////////////
    let (framebuffer, mut logger) = {
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

        (framebuffer, logger)
    };

    unsafe {
        LOGGER = Some(&mut logger);
    }

    writeln!(
        logger,
        "EfiLogger initialized, using framebuffer at {:X}, len {:X}\r",
        framebuffer.address, framebuffer.len
    )
    .unwrap();

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Step 2: Load kernel into memory                                                            //
    ////////////////////////////////////////////////////////////////////////////////////////////////
    let mut kernel_data = load_kernel(handle, st).expect("Unable to load kernel");
    let kernel_file = ElfFile::read(kernel_data.as_mut_slice());

    if !kernel_file.is_valid() {
        panic!(
            "Can not load kernel: File is corrupted, it is not of type ELF\r\nKernel Length: {:?}",
            kernel_file.data().len()
        )
    }

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
                kernel_file.data().as_ptr() as u64 + offset,
                v_addr,
                mem_size
            )
            .unwrap();
        });
    let kernel_len = kernel_file.load_segments_len();

    //Prepare page table
    //let page_table = init_page_table(&kernel_file);

    //Retrieve RSDP
    //TODO: Read RSDP

    let mut kargs = Box::new(KernelArgs {
        kernel_ptr: 0x100000 as *const u8,
        kernel_len: 0,
        rsd_ptr: &0u8,
        memory_map: &0u8,
        memory_map_size: 0,
        memory_map_type: 0,
        framebuffer,
    });

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Step 3: Prepare stack                                                                      //
    ////////////////////////////////////////////////////////////////////////////////////////////////
    let stack_size: usize = 128 * 1024;
    let mut stack: Vec<u8> = Vec::new();
    stack.resize(stack_size, 0);
    let stack_ptr = stack.as_ptr() as usize + stack_size;

    //Prepare identity paging
    //let page_tables = identity_paging();

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Step 4: Exit the boot services and get memory map                                          //
    ////////////////////////////////////////////////////////////////////////////////////////////////
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
        kargs.as_ref() as *const KernelArgs as u64
    )
    .unwrap();

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Step 5: Copy the kernel to 0x100000 static location and apply relocations                  //
    ////////////////////////////////////////////////////////////////////////////////////////////////
    writeln!(logger, "Allocating kernel: {}\r", kernel_len).unwrap();
    //Write the kernel to 0x100000 in physical memory
    let memory_location = unsafe { slice::from_raw_parts_mut(0x100000 as *mut u8, kernel_len) };
    kernel_file.load(memory_location);
    kernel_file.relocate(memory_location);

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Step 1: Call the kernel                                                                    //
    ////////////////////////////////////////////////////////////////////////////////////////////////
    //Load the main function into variable
    let kernel_main: unsafe extern "sysv64" fn(*const KernelArgs) -> ! =
        unsafe { mem::transmute(memory_location.as_ptr().add(kernel_file.entrypoint())) };

    unsafe {
        writeln!(logger, "Calling kernel\r").unwrap();

        //asm!("mov cr3, {}", in(reg) (page_tables.as_ptr() as u64) << 12);

        call_kernel(kernel_main, stack_ptr, kargs.as_ref())
        //writeln!(logger, "Kernel returned: {:X}\r", returnval).unwrap();
    }
    loop {}
}

#[allow(unsafe_code)]
pub unsafe fn call_kernel(
    kmain: unsafe extern "sysv64" fn(*const KernelArgs) -> !,
    stack_ptr: usize,
    args: &KernelArgs,
) -> ! {
    asm!("mov rsp, {}", in(reg) stack_ptr);

    kmain(args);
}

#[allow(unsafe_code)]
pub fn load_kernel(handle: EfiHandle, st: &SystemTable) -> Result<Vec<u8>, EfiStatus> {
    let mut logger = unsafe { &mut *LOGGER.unwrap() };
    //logger.log("NightOS Bootloader (v0.0.1)\r\n");

    //logger.log("Loading kernel from /kernel\r\n");
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

    let file = unsafe { (*root).open("kernel", 1, 0).expect("Unable to open file") };
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
pub fn identity_paging() -> Vec<u64> {
    let mut page_table = Vec::new();
    page_table.resize(2048, 0);

    let mut address_pdp = 0;
    unsafe {
        address_pdp = page_table.as_ptr().add(1024) as u64;
    }

    let pml4_entry: u64 = PageMapTableBuilder::from(0)
        .address(address_pdp)
        .present(true)
        .into();

    let mut page = page_table.get_mut(0).unwrap();
    *page = pml4_entry;

    for i in 0..8usize {
        let entry: u64 = PageMapTableBuilder::from(0)
            .address((i * 1024 * 1024 * 1024) as u64)
            .execute_disable(false)
            .page_size(true)
            .present(true)
            .into();

        let mut page = page_table.get_mut(1024 + i).unwrap();
        *page = entry;
    }

    page_table
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
    kernel_ptr: *const u8,
    kernel_len: u64,

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
