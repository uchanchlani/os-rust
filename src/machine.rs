
pub const KERNEL_SPACE: u64 = 0x4_00000; // 4MB
pub const KERNEL_VIR_START: u64 = 0o1_000_000_0000 - 0o1_000_0000; // 1GB - 2MB
pub const KERNEL_PHY_START: u64 = 0o1_000_0000; // 2MB
pub const PAGE_SIZE: u64 = 0o1_0000; // 4KB
pub const PAGE_OFFSET_BITS : u8 = 12;
pub const HEAP_START : u64 = 0o1_000_000_0000;
pub const HEAP_SIZE : u64 = 0o1_000_000_0000;
pub const L4_PAGE_TABLE_VADDR : u64 = 0o1_77777_777_777_777_777_0000;
pub const L3_PAGE_TABLE_VADDR : u64 = 0o1_77777_777_777_777_000_0000;
pub const L2_PAGE_TABLE_VADDR : u64 = 0o1_77777_777_777_000_000_0000;

pub type CFunc = extern "C" fn();