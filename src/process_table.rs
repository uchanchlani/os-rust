use x86_64::{
    registers::control::Cr3,
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

static mut CURR_PROCESS_TABLE: *mut MyProcess = 0x0 as *mut MyProcess;
static mut NEXT_PROCESS: *mut MyProcess = 0x0 as *mut MyProcess;

#[allow(dead_code)]
fn get_curr_process_table() -> &'static MyProcess {
    unsafe {
        & (*CURR_PROCESS_TABLE)
    }
}

#[allow(dead_code)]
fn get_curr_process_table_mut() -> &'static mut MyProcess {
    unsafe {
        &mut (*CURR_PROCESS_TABLE)
    }
}

#[allow(dead_code)]
fn set_curr_process_table(pt : &mut MyProcess) {
    unsafe {
        CURR_PROCESS_TABLE = &mut (*pt)
    }
}

#[allow(dead_code)]
pub fn set_next_process(pt : &mut MyProcess) {
    unsafe {
        NEXT_PROCESS = &mut (*pt)
    }
}

static mut NEXT_PROCESS_ID : u16 = 0;

fn faa_next_proc_id() -> u16 {
    unsafe {
        NEXT_PROCESS_ID+=1;
        NEXT_PROCESS_ID
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MyProcess {
    esp : u64,
    pg_dir_phy : PhysAddr,
    page_directory : VirtAddr,
    process_id : u16,
    stack_size : u16,
    started : bool,
    terminated : bool,
    pub vm_pool : &'static mut VMPool,
    next : *mut MyProcess
}

fn get_page_table_from_addr(addr : u64) -> &'static mut PageTable {
    unsafe {
        &mut *(addr as *mut PageTable)
    }
}

impl MyProcess {
    fn construct_page_table(& self) {
        let pg_table_addr = self.page_directory;
        let dest_table = get_page_table_from_addr(pg_table_addr.as_u64());
        let source_table = get_page_table_from_addr(crate::machine::L4_PAGE_TABLE_VADDR);
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
        let source_table1 = get_page_table_from_addr(crate::machine::L3_PAGE_TABLE_VADDR);
        let dest_table1 = get_page_table_from_addr(p3_addr_vir.as_u64());
        dest_table1.zero();
        dest_table1[0].set_addr(source_table1[0].addr(), source_table1[0].flags());

    }

    fn push(&mut self, val : u64) {
        self.esp -= 8;
        unsafe {
            *(self.esp as *mut u64) = val;
        }
    }

    fn construct_stack(&mut self, p_func_ptr : crate::machine::CFunc, _stack_size: u64) {
        let old_cr3 = Cr3::read();
        let old_pt = get_curr_process_table_mut();
        unsafe {
            set_curr_process_table(self);
            Cr3::write(PhysFrame::containing_address(self.pg_dir_phy), old_cr3.1);
        }

        let stack_frame = self.vm_pool.allocate(_stack_size as usize);
        self.stack_size = _stack_size as u16;
        self.esp = stack_frame.unwrap().as_u64() + _stack_size as u64;

        let pe = ((process_end as crate::machine::CFunc) as *const extern "C" fn()) as u64;
        self.push(pe);
        let pf = (p_func_ptr as *const extern "C" fn()) as u64;
        self.push(pf);

        let curr_esp = self.esp;


        self.push(0 as u64); // ss register
        self.push(curr_esp); // rsp register
        self.push(0 as u64); // rflags
        let cs = crate::gdt::get_cs();
        self.push(cs); // CS
        let ps = ((process_start as crate::machine::CFunc) as *const extern "C" fn()) as u64;
        self.push(ps);

//        for i in 0..16 {
//            self.push(0 as u64);
//        } // 16 general purpose registers

        unsafe {
            Cr3::write(old_cr3.0, old_cr3.1);
            set_curr_process_table(old_pt);
        }
    }

    pub fn new(p_func_ptr : crate::machine::CFunc) -> &'static mut Self {
        crate::interrupts::disable_interrupts();

        // init process instance
        let frame = crate::memory::get_frame(true, false);
        let fr_addr = frame.unwrap().start_address().as_u64();
        let my_process : &mut MyProcess = unsafe {&mut *(fr_addr as *mut MyProcess)};

        my_process.pg_dir_phy = crate::memory::get_frame(true, true).unwrap().start_address();
        my_process.page_directory = crate::memory::transform_kernel_to_vir(my_process.pg_dir_phy);

        my_process.construct_page_table();

        // VMPool Starts at 1GB and is of size 1GB
        let vm_page = crate::memory::get_frame(true, false).unwrap().start_address().as_u64();
        my_process.vm_pool = unsafe {&mut *(vm_page as *mut VMPool)};
        my_process.vm_pool.new(
            crate::machine::HEAP_START,
            crate::machine::HEAP_SIZE);


        my_process.next = 0x0 as *mut MyProcess;
        my_process.process_id = faa_next_proc_id();

        my_process.construct_stack(p_func_ptr, 8192);

        crate::interrupts::enable_interrupts();
        my_process
    }

    pub fn load_page_table(&'static mut self) -> &'static mut Self {
        set_curr_process_table(self);
        unsafe {
            Cr3::write(PhysFrame::containing_address(self.pg_dir_phy), Cr3::read().1);
        }
        self
    }

    pub fn handle_fault(_addr : VirtAddr) -> bool {
        unsafe {

            let vm_pool = &get_curr_process_table().vm_pool;
            if vm_pool.is_legitimate(_addr) {
                let level_4_table_ptr = crate::machine::L4_PAGE_TABLE_VADDR as *mut PageTable;
                let level_4_table = &mut *level_4_table_ptr;
                let mut rptr = RecursivePageTable::new(level_4_table).unwrap();
                let option = crate::memory::get_frame(false, false);
                if option.is_none() {
                    return false;
                }
                let frame = option.unwrap();
                let result = rptr.map_to(Page::containing_address(_addr), frame, Flags::PRESENT | Flags::WRITABLE, crate::memory::get_frame_pool_mut(true));
                result.is_ok()
//                true
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
    let source_table = &mut *(table as *mut PageTable);
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

    if !MyProcess::handle_fault(addr) {
        println!("EXCEPTION: PAGE FAULT");
        println!("Accessed Address: {:?}", addr);
        println!("{:#?}", stack_frame);
        panic!();
    }
}

#[naked]
pub extern "C" fn process_switch_to() {
    unsafe {
        asm!("mov $0, $1
              mov rbx, $0"
        : "=r"(CURR_PROCESS_TABLE)
        : "r"(NEXT_PROCESS)
        ::"volatile", "intel");

        asm!("mov rax, [rbx+8]
              mov rsp, [rbx]
              mov cr3, rax
              iretq"
        ::::"volatile", "intel");
    }
}

extern "C" fn process_start() {
    unsafe {
        get_curr_process_table_mut().started = true;
        crate::interrupts::enable_interrupts();
    }
}

extern "C" fn process_end() {
    unsafe {
        crate::hlt_loop();
    }
}