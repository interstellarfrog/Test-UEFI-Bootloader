#![no_std] // Disable Linking Of Rust Standard Lib
#![no_main] // Disable Normal Main Based On C Runtime
#![feature(alloc_error_handler)]

use core::{panic::PanicInfo, alloc::Layout};
use uefi::prelude::entry;
use core::fmt::Write;
use uefi::table::cfg;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{MemoryDescriptor, MemoryType};
use core::{mem, slice};

extern crate alloc;

use alloc::vec::Vec;

#[panic_handler]
fn panic(_panic_info: &PanicInfo) -> ! {
    loop {}
}

static KERNEL: &[u8] = include_bytes!("../../interstellar_os/target/x86_64-interstellar_os/release/interstellar_os.d");


#[entry]
fn efi_main(
    image: uefi::Handle, 
    mut system_table: uefi::table::SystemTable<uefi::table::Boot>,
) -> uefi::Status {

    unsafe{ uefi::alloc::init(system_table.boot_services()) }
    
    let mut v = Vec::new();
    v.push(1);
    v.push(2);


    let stdout = system_table.stdout();
    stdout.clear().unwrap();

    writeln!(stdout, "v = {:?}", v).unwrap();

    
    writeln!(stdout, "Hello World!").unwrap();


    let mut config_entries = system_table.config_table().iter();
    let rsdp_addr = config_entries
        .find(|entry| matches!(entry.guid, cfg::ACPI_GUID | cfg::ACPI2_GUID))
        .map(|entry| entry.address);
    writeln!(stdout, "rsdp addr: {:?}", rsdp_addr).unwrap();

    let protocol = system_table.boot_services().locate_protocol::<GraphicsOutput>()
        .unwrap().unwrap();
    let gop = unsafe { &mut *protocol.get() };
    writeln!(stdout, "current gop mode: {:?}", gop.current_mode_info()).unwrap();
    writeln!(stdout, "framebuffer at: {:#p}", gop.frame_buffer().as_mut_ptr()).unwrap();

    let mmap_storage = {
        let max_mmap_size = system_table.boot_services().memory_map_size()
            + 8 * mem::size_of::<MemoryDescriptor>();
        let ptr = system_table
            .boot_services()
            .allocate_pool(MemoryType::LOADER_DATA, max_mmap_size)
            .unwrap();
        unsafe { slice::from_raw_parts_mut(ptr.unwrap(), max_mmap_size) }
    };
    
    uefi::alloc::exit_boot_services();
    let (system_table, memory_map) = system_table.exit_boot_services(image, mmap_storage)
        .unwrap().unwrap();
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    panic!("out of memory")
}