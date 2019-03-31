#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

use blog_os::{exit_qemu, serial_println};
use bootloader::{bootinfo::BootInfo, entry_point};
use core::panic::PanicInfo;

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

    blog_os::memory::init_frame_allocator(&boot_info.memory_map);

    let frame = blog_os::memory::get_frame(true, false);
    let mut test_mem : [u32; 1024] = unsafe {*(frame.unwrap().start_address().as_u64() as *mut [u32; 1024])};

    for i in 0..1024 {
        test_mem[i] = i as u32;
    }

    for i in 0..1024 {
        if test_mem[i] != i as u32 {
            panic!();
        }
    }

    serial_println!("ok");
//    serial_println!("{}", core::mem::size_of::<blog_os::memory::SimpleFramePool>());
    unsafe { exit_qemu(); }
    blog_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");

    serial_println!("{}", info);

    unsafe { exit_qemu(); }
    blog_os::hlt_loop();
}

