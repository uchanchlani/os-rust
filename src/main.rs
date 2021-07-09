#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

use core::panic::PanicInfo;
use blog_os::println;
use blog_os::serial_println;
use bootloader::{bootinfo::BootInfo, entry_point};
use x86_64::{
    structures::{
        paging::{
            PhysFrame,
            Size4KiB,
            PageTable,
            RecursivePageTable
        }
    }
};

entry_point!(kernel_main);

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    blog_os::hlt_loop();
}

#[cfg(not(test))]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    blog_os::gdt::init();
    blog_os::interrupts::init_idt();
    blog_os::interrupts::init_pics();
    blog_os::interrupts::enable_interrupts();

    use blog_os::memory::{self};

    let recursive_page_table = unsafe {
//        let cr3 = x86_64::registers::control::Cr3::read();


        memory::init_frame_allocator(&boot_info.memory_map);

        let level_4_table_ptr = boot_info.p4_table_addr as *mut PageTable;
        let level_4_table = &mut *level_4_table_ptr;
        RecursivePageTable::new(level_4_table).unwrap()
    };

    let frame = x86_64::registers::control::Cr3::read();
    let x : PhysFrame<Size4KiB> = frame.0;
    println!("{:x}", boot_info.p4_table_addr);
    serial_println!("{:x}", boot_info.p4_table_addr);

    println!("It did not crash!");
    blog_os::hlt_loop();
}

