#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(naked_functions)]

use blog_os::{exit_qemu, serial_println};
use bootloader::{bootinfo::BootInfo, entry_point};
use core::panic::PanicInfo;
use blog_os::process_table::MyProcess;

entry_point!(kernel_main);

#[cfg(not(test))]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    blog_os::gdt::init();
    blog_os::interrupts::init_idt();
    blog_os::interrupts::init_pics();
    blog_os::interrupts::enable_interrupts();

    blog_os::memory::init_frame_allocator(&boot_info.memory_map);

    let my_process : &'static mut MyProcess = MyProcess::new(process_function as blog_os::machine::CFunc);
    blog_os::process_table::set_next_process(my_process);
    blog_os::process_table::process_switch_to();

    panic!();
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

extern "C" fn process_function() {
    serial_println!("ok");
    unsafe { exit_qemu(); }
    blog_os::hlt_loop();
}

