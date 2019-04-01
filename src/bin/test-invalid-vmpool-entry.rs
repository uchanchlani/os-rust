#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

use blog_os::{exit_qemu, serial_println};
use bootloader::{bootinfo::BootInfo, entry_point};
use core::panic::PanicInfo;
use blog_os::process_table::ProcessTable;
//use core::mem::size_of;

entry_point!(kernel_main);

//#[cfg(not(test))]
//#[no_mangle]
//pub extern "C" fn _start() -> ! {
#[cfg(not(test))]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    blog_os::gdt::init();
    blog_os::interrupts::init_idt();
    blog_os::interrupts::init_pics();
    blog_os::interrupts::enable_interrupts();

    let _cr3 = x86_64::registers::control::Cr3::read();

    blog_os::process_table::fixCr3();
    blog_os::memory::init_frame_allocator(&boot_info.memory_map);

    let mut page_table : &'static mut ProcessTable = ProcessTable::new();
    page_table = page_table.load();
    let x = &mut page_table.vm_pool;
    let addr = x.allocate(core::mem::size_of::<[u64; 10000]>()).unwrap();
    let test_mem : &mut [u64; 10000] = unsafe {&mut *(addr.as_u64() as *mut [u64; 10000])};

    for i in 0..10000 {
        test_mem[i] = i as u64;
    }

    for i in 0..10000 {
        if test_mem[i] != i as u64 {
            panic!();
        }
    }

    x.release(addr);
    let test_mem : &mut [u64; 10000] = unsafe {&mut *(addr.as_u64() as *mut [u64; 10000])}; // this should panic
    test_mem[0] = 0;

    serial_println!("failed");

    unsafe { exit_qemu(); }
    blog_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("ok");

    unsafe { exit_qemu(); }
    blog_os::hlt_loop();
}

