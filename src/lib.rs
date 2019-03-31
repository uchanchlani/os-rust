
#![cfg_attr(not(test), no_std)] // don't link the Rust standard library
#![feature(abi_x86_interrupt)]
#![feature(asm)]

pub mod machine;
pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod page_table;
pub mod vm_pool;

pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn asdf() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
pub fn asdf1() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}