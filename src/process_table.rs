//#![feature(const_raw_ptr_deref)]

use x86_64::{
    registers::control::Cr3,
    registers::control::Cr3Flags,
    registers::control::Cr2,
    structures::{
        idt::{
            ExceptionStackFrame,
            PageFaultErrorCode
        },
        paging::{
            PhysFrame,
            RecursivePageTable,
            PageTable,
            PageTableFlags as Flags
        }
    },
    VirtAddr,
    PhysAddr,
};
use crate::{
    println,
    serial_println,
    vm_pool::VMPool
};
use x86_64::structures::paging::{Mapper, Page};

pub static mut CURR_PROCESS_TABLE: *mut ProcessTable = 0x0 as *mut ProcessTable;

#[repr(C)]
pub struct ProcessTable {
    pub pg_dir_phy : PhysAddr,
    pub page_directory : VirtAddr,
    pub vm_pool : &'static mut VMPool
}

pub fn fixCr3() {
    unsafe {
        let frame = Cr3::read();
        Cr3::write(frame.0, Cr3Flags::PAGE_LEVEL_WRITETHROUGH);
    }
}

impl ProcessTable {
    fn construct_page_table(& self) {
        let pg_table_addr = self.page_directory;
        let mut dest_table = unsafe {&mut *(pg_table_addr.as_u64() as *mut PageTable)};
        let source_table = unsafe{&mut *(crate::machine::L4_PAGE_TABLE_VADDR as *mut PageTable)};
        let p3_frame = crate::memory::get_frame(true, true).unwrap();
        dest_table.zero();
        dest_table[0].set_addr(p3_frame.start_address(), Flags::PRESENT | Flags::WRITABLE);
        dest_table[511].set_addr(self.pg_dir_phy, Flags::PRESENT | Flags::WRITABLE);

        for i in 1..511 {
            if source_table[i].is_unused() {
                continue;
            }
            dest_table[i].set_addr(source_table[i].addr(), source_table[0].flags());
        }

        let p3_addr_vir = crate::memory::transform_kernel_to_vir(p3_frame.start_address());
        let source_table1 = unsafe{&mut *(crate::machine::L3_PAGE_TABLE_VADDR as *mut PageTable)};
        let mut dest_table1 = unsafe {&mut *(p3_addr_vir.as_u64() as *mut PageTable)};
        dest_table1.zero();
        dest_table1[0].set_addr(source_table1[0].addr(), source_table1[0].flags());

    }

    pub fn new() -> &'static mut Self {
        let frame = crate::memory::get_frame(true, false);
        let page_table : &mut ProcessTable = unsafe {&mut *(frame.unwrap().start_address().as_u64() as *mut ProcessTable)};

        page_table.pg_dir_phy = crate::memory::get_frame(true, true).unwrap().start_address();
        page_table.page_directory = crate::memory::transform_kernel_to_vir(page_table.pg_dir_phy);

        page_table.construct_page_table();

        // VMPool Starts at 1GB and is of size 1GB
        let vm_page = crate::memory::get_frame(true, false).unwrap().start_address().as_u64();
        page_table.vm_pool = unsafe {&mut *(vm_page as *mut VMPool)};
        page_table.vm_pool.new(
            crate::machine::HEAP_START,
            crate::machine::HEAP_SIZE);

        page_table
    }

    pub fn load(&'static mut self) -> &'static mut Self {
        unsafe {
            CURR_PROCESS_TABLE = &mut (*self);
            Cr3::write(PhysFrame::containing_address(self.pg_dir_phy), Cr3::read().1);
        }
        self
    }

    pub fn handle_fault(_addr : VirtAddr) -> bool {
        unsafe {
            let vm_pool = &(*CURR_PROCESS_TABLE).vm_pool;
            if vm_pool.is_legitimate(_addr) {
                let level_4_table_ptr = crate::machine::L4_PAGE_TABLE_VADDR as *mut PageTable;
                let level_4_table = &mut *level_4_table_ptr;
                let mut rptr = RecursivePageTable::new(level_4_table).unwrap();
                let option = crate::memory::get_frame(false, false);
                if option.is_none() {
                    return false;
                }
                let frame = option.unwrap();
                rptr.map_to(Page::containing_address(_addr), frame, Flags::PRESENT | Flags::WRITABLE, &mut *(crate::memory::SYSTEM_FRAME_POOL));
                true
            } else {
                false
            }
        }
    }

    pub fn free_page(_addr : VirtAddr) {
        unsafe {
            let level_4_table_ptr = crate::machine::L4_PAGE_TABLE_VADDR as *mut PageTable;
            let level_4_table = &mut *level_4_table_ptr;
            let mut rptr = RecursivePageTable::new(level_4_table).unwrap();
            let unmap_result = rptr.unmap(Page::containing_address(_addr));
            if unmap_result.is_ok() {
                let unmap = unmap_result.unwrap();
                let frame = unmap.0;
                let flush = unmap.1;
                crate::memory::free_frame(frame);
                flush.flush();
            }
        };
    }

    pub fn get_vm_ref(&'static mut self) -> &mut VMPool {
        &mut self.vm_pool
    }

}

pub unsafe fn print_pg_tables(table: u64) {
    let source_table = unsafe{&mut *(table as *mut PageTable)};
    for i in 0..512 {
        let entry = &source_table[i];
        if !entry.is_unused() {
            serial_println!("Entry {}: {:?}", i, entry);
        }
    }
}

//unsafe fn get_rptr() -> RecursivePageTable {
//    let level_4_table_ptr = crate::machine::L4_PAGE_TABLE_VADDR as *mut PageTable;
//    let level_4_table = &mut *level_4_table_ptr;
//    let mut rptr = RecursivePageTable::new(level_4_table).unwrap();
//    rptr
//}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: PageFaultErrorCode,
) {
    let addr = Cr2::read();

    if !ProcessTable::handle_fault(addr) {
        println!("EXCEPTION: PAGE FAULT");
        println!("Accessed Address: {:?}", addr);
        println!("{:#?}", stack_frame);
        panic!();
    }
}

