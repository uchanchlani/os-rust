#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

use core::panic::PanicInfo;
use blog_os::println;
use bootloader::{bootinfo::BootInfo, entry_point};

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

    use blog_os::memory::{self, translate_addr};

    let mut recursive_page_table = unsafe {
        memory::init(boot_info.p4_table_addr as usize)
    };

    let mut frame_allocator = memory::init_frame_allocator(&boot_info.memory_map);
    blog_os::memory::create_example_mapping(&mut recursive_page_table, &mut frame_allocator);

    println!("0xdeadbeaf900 -> {:?}", translate_addr(0xdeadbeaf900, &recursive_page_table));
    unsafe { (0xdeadbeaf900 as *mut u64).write_volatile(0xf021f077f065f04e)};

    println!("It did not crash!");
    blog_os::hlt_loop();
}

