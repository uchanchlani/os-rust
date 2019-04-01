use x86_64::{
    structures::{
        paging::{
            PageTable,
            FrameAllocator,
            PhysFrame,
            Size4KiB
        }
    },
    VirtAddr,
    PhysAddr
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
// @TODO
use crate::serial_println;
use crate::serial_print;
use x86_64::structures::paging::FrameDeallocator;

//const KERNEL_SPACE: u64 = 0x4_00000; // 4MB
//const KERNEL_VIR_START: u64 = 0o1_000_000_0000 - 0o1_000_0000; // 1GB - 2MB
//const KERNEL_PHY_START: u64 = 0o1_000_0000; // 2MB
//const PAGE_SIZE: u64 = 0o1_0000; // 4KB

pub static mut SYSTEM_FRAME_POOL : *mut SimpleFramePool = (0x0 as *mut SimpleFramePool);
pub static mut USER_FRAME_POOL : *mut SimpleFramePool = (0x0 as *mut SimpleFramePool);

pub struct SimpleFramePool { // manages frames from i (510*8*8) till i+1 (510*8*8)
    start_frame: u64,
    next : *mut SimpleFramePool,
//    frames : *mut u8
    frames: [u8; 4080] // because 8 u8s are gone for the start frame
}

#[allow(dead_code)]
impl SimpleFramePool {
    fn init(&mut self, start_frame : u64) {
        self.start_frame = start_frame;
        self.frames = [0b0000_0000; 4080];
        self.next = 0x0 as *mut SimpleFramePool;
    }

    fn add_frame_pool(&mut self, next : *mut SimpleFramePool) {
        self.next = next;
    }

    fn mark_free_block(mut old_value : u8, start : u8, end : u8) -> u8 {
        let mut free_blocks : u8 = 0b1111_1111;
        if start + 8 - end > 0 {
            free_blocks >>= start + 8 - end;
        }
        if 8 - end > 0 {
            free_blocks <<= 8 - end;
        }
        old_value |= free_blocks;
        old_value
    }

    fn mark_used_block(mut old_value : u8, start : u8, end : u8) -> u8 {
        let mut free_blocks : u8 = 0b1111_1111;
        if start + 8 - end > 0 {
            free_blocks >>= start + 8 - end;
        }
        if 8 - end > 0 {
            free_blocks <<= 8 - end;
        }
        free_blocks = !free_blocks;
        old_value &= free_blocks;
        old_value
    }

    fn is_full_block(value : u8) -> bool {
        value == 0b0000_0000
    }

    fn get_first_free_block(value : u8) -> u8 {
        let mut ret : u8 = 0;
        let mut mask : u8 = 0b1000_0000;
        while ret < 8 {
            if mask & value != 0 {
                break;
            }
            ret += 1;
            mask >>= 1;
        }
        return ret;
    }

    fn mark_free(&mut self, mut start_frame : u64, end_frame : u64) {
        let end_byte = end_frame / 8;
        while start_frame < end_frame {
            let start_byte = start_frame / 8;
            let start_offset  : u8 = (start_frame % 8) as u8;
            let end_offset  : u8 = if start_byte == end_byte {
                (end_frame % 8) as u8
            } else {
                8 as u8
            };
            self.frames[start_byte as usize] = SimpleFramePool::mark_free_block(self.frames[start_byte as usize], start_offset, end_offset);
            start_frame += end_offset as u64 - start_offset as u64;
        }
    }

    fn mark_used(&mut self, mut start_frame : u64, end_frame : u64) {
        let end_byte = end_frame / 8;
        while start_frame < end_frame {
            let start_byte = start_frame / 8;
            let start_offset  : u8 = (start_frame % 8) as u8;
            let end_offset  : u8 = if start_byte == end_byte {
                (end_frame % 8) as u8
            } else {
                8 as u8
            };
            self.frames[start_byte as usize] = SimpleFramePool::mark_used_block(self.frames[start_byte as usize], start_offset, end_offset);
            start_frame += end_offset as u64 - start_offset as u64;
        }
    }

    fn find_free_frame(&mut self) -> Option<PhysFrame> {
        for i in 0..4080 {
            let _byte : u8 = self.frames[i];
            if SimpleFramePool::is_full_block(_byte) {
                continue;
            }
            let offset = SimpleFramePool::get_first_free_block(_byte);
            self.mark_used(i as u64 * 8 + offset as u64, i as u64 * 8 + offset as u64 + 1);
            return Some(PhysFrame::containing_address(PhysAddr::new(self.start_frame + crate::machine::PAGE_SIZE * (i as u64 * 8 + offset as u64))));
        };

        return None;
    }

    pub fn free_frame(frame : PhysFrame) {
        let frame_addr : u64 = frame.start_address().as_u64();
        unsafe {
            let mut fp: *mut SimpleFramePool = SYSTEM_FRAME_POOL;
            loop {
                if (*fp).next == 0x0 as *mut SimpleFramePool || (*(*fp).next).start_frame > frame_addr {
                    (*fp).mark_free((frame_addr - (*fp).start_frame) / crate::machine::PAGE_SIZE, (frame_addr - (*fp).start_frame) / crate::machine::PAGE_SIZE + 1);
                    break
                }
                fp = (*fp).next;
            }
        }
    }

    pub fn print_frame_map(& self) {
        let x = self.frames;
        for i in 155..170 {
            serial_print!("{:08b} ", x[i]);
        }
        serial_println!("");
    }
}

impl FrameAllocator<Size4KiB> for SimpleFramePool {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.find_free_frame()
    }
}

impl FrameDeallocator<Size4KiB> for SimpleFramePool {
    fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        SimpleFramePool::free_frame(frame);
    }
}

pub fn get_frame(kernel: bool, raw: bool) -> Option<PhysFrame> {
    if kernel {
        unsafe {
            let frame = (*SYSTEM_FRAME_POOL).allocate_frame();
            if frame.is_none() {
                return None;
            };
            if raw {
                frame
            } else {
                let virt_addr = transform_kernel_to_vir(frame.unwrap().start_address());
                Some(PhysFrame::containing_address(PhysAddr::new(virt_addr.as_u64())))
            }
        }
    } else {
        unsafe { (*USER_FRAME_POOL).allocate_frame()}
    }
}

pub fn free_frame(frame : PhysFrame) {
    SimpleFramePool::free_frame(frame)
}

pub fn print_frame_map() {
    unsafe {(*SYSTEM_FRAME_POOL).print_frame_map()};
}

//#[allow(dead_code)]
pub fn transform_kernel_to_vir(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64() - crate::machine::KERNEL_PHY_START + crate::machine::KERNEL_VIR_START)
}

pub fn init_frame_allocator(
    memory_map: &'static MemoryMap) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let mut i1 = 0x0;

    let pt_0mb : &mut PageTable = unsafe { &mut *(0o1_77777_777_000_000_000_0000 as *mut PageTable)}; // first 2 MB level 2 page
    for i in 0..512 {
        pt_0mb[i].set_addr(PhysAddr::new(i1), Flags::PRESENT | Flags::WRITABLE);
        i1 = i1 + 0o1_0000; // 4KB
    }

    let pt_p2 : &mut PageTable = unsafe { &mut *(0o1_77777_777_777_000_000_0000 as *mut PageTable)};
    pt_p2[511].set_addr(PhysAddr::new(0x2_00000), Flags::PRESENT | Flags::WRITABLE | Flags::HUGE_PAGE);

    // get usable regions from memory map
    let regions = memory_map
        .iter()
        .filter(|r| r.region_type == MemoryRegionType::Usable);

    // use address range less than 4 MB for kernel frame pool
    for region in regions {
        if region.range.end_addr() <= crate::machine::KERNEL_SPACE {
            let sys_frame_addr = transform_kernel_to_vir(PhysAddr::new(region.range.start_addr()));
            unsafe {
                SYSTEM_FRAME_POOL = sys_frame_addr.as_u64() as *mut SimpleFramePool;
                (*SYSTEM_FRAME_POOL).init(crate::machine::KERNEL_PHY_START);
                (*SYSTEM_FRAME_POOL).mark_free(
                    (region.range.start_addr() - crate::machine::KERNEL_PHY_START)/crate::machine::PAGE_SIZE + 1,
                    (region.range.end_addr() - crate::machine::KERNEL_PHY_START)/crate::machine::PAGE_SIZE);
            };
        } else {
            unsafe {
                let user_frame_addr = get_frame(true, false).unwrap().start_address();
                USER_FRAME_POOL = user_frame_addr.as_u64() as *mut SimpleFramePool;
                (*SYSTEM_FRAME_POOL).add_frame_pool(USER_FRAME_POOL);
                (*USER_FRAME_POOL).init(crate::machine::KERNEL_SPACE);
                (*USER_FRAME_POOL).mark_free(
                    (region.range.start_addr() - crate::machine::KERNEL_SPACE)/crate::machine::PAGE_SIZE,
                    (region.range.end_addr() - crate::machine::KERNEL_SPACE)/crate::machine::PAGE_SIZE);
            };
        }
    }
}
