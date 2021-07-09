use crate::{
    process_table::MyProcess
};
use x86_64::VirtAddr;
use core::fmt::{Formatter, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VMPoolEntry {
    start : VirtAddr,
    size : u64 // number of pages it spans
}

impl VMPoolEntry {
    fn end(& self) -> VirtAddr {
        self.start + (self.size * crate::machine::PAGE_SIZE)
    }

    fn contained(& self, addr : VirtAddr) -> bool {
        addr >= self.start && addr < self.end()
    }

    fn update_val(&mut self, start : VirtAddr, size : u64) {
        self.start = start;
        self.size = size;
    }

    fn free_entry(&mut self) {
        self.update_val(VirtAddr::new(0), 0 as u64);
    }
}

//#[derive(Debug)]
#[repr(C)]
pub struct VMPool {
    start_addr: VirtAddr,
    pool_size : u64,
    entries : [VMPoolEntry; 254]
}

impl VMPool {
    pub fn new(&mut self,
        _base_addr : u64,
        _size : u64/*,
        _page_table : *mut ProcessTable*/) {
        self.start_addr = VirtAddr::new(_base_addr);
        self.pool_size = _size >> crate::machine::PAGE_OFFSET_BITS;
        self.entries = [VMPoolEntry{start:VirtAddr::new(0), size:0}; 254];
    }

    fn end_addr(& self) -> VirtAddr {
        self.start_addr + (self.pool_size * crate::machine::PAGE_SIZE)
    }

    fn contained(& self, addr : VirtAddr) -> bool {
        addr >= self.start_addr && addr < self.end_addr()
    }

    pub fn is_legitimate(& self, addr: VirtAddr) -> bool {
        if !self.contained(addr) {
            return false;
        }
        for entry in self.entries.iter() {
            if entry.start.as_u64() == 0 {
                continue;
            } else {
                if entry.contained(addr) {
                    return true;
                }
            }
        }
        false
    }

    pub fn allocate(&mut self, size : usize) -> Option<VirtAddr> {
        let mut size = size as u64 + crate::machine::PAGE_SIZE - 1;
        size >>= crate::machine::PAGE_OFFSET_BITS;
        let mut min_free = self.start_addr;
        for entry in self.entries.iter() {
            if entry.start.as_u64() == 0 {
                continue;
            } else {
                min_free = if min_free < entry.end() {
                    entry.end()
                } else {
                    min_free
                };
            }

            if !self.contained(min_free+(size*crate::machine::PAGE_SIZE)) {
                return None;
            }
        }
        for entry in self.entries.iter_mut() {
            if entry.start.as_u64() == 0 {
                entry.update_val(min_free, size);
                break;
            }
        }
        Some(min_free)
    }

    pub fn release(&mut self, addr: VirtAddr) {
        for entry in self.entries.iter_mut() {
            if entry.start.as_u64() == 0 {
                continue;
            } else if entry.start == addr {
                for i in 0..entry.size {
                    let mut free_addr : u64 = entry.start.as_u64();
                    free_addr += i*crate::machine::PAGE_SIZE;
                    MyProcess::free_page(VirtAddr::new(free_addr));
                }
                entry.free_entry();
                break;
            }
        }
    }
}

impl core::fmt::Debug for VMPool {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "")
    }
}