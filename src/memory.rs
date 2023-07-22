use std::mem;
use std::ptr;
use std::u8;

use elfloader::VAddr;

// type VAddr = u64;
// type PAddr = u64;

#[derive(Debug, Copy, Clone)]
pub struct RAM {
    pub base_address: VAddr,
    pub size: usize,
    data: *mut u8,
}

pub trait MemoryCellType {}
impl MemoryCellType for u8 {}

#[allow(unused_variables)]
pub trait MemoryOperations<T, T2: MemoryCellType>: std::fmt::Debug {
    fn read8(&self, addr: VAddr) -> Option<T2>;
    fn write8(&mut self, addr: VAddr, value: T2) -> bool;

    fn read32(&self, addr: VAddr) -> Option<u32>;
    fn write32(&mut self, addr: VAddr, value: u32);

    //    fn add_segment(&mut self, base_address: VAddr, size: usize);
}

pub trait RAMOperations<T>: MemoryOperations<T, u8> {
    fn read_32(&self, addr: VAddr) -> Option<u32> {
        // @FIXME: We allow 16-bit aligned access, because instruction fetches are 16-bit aligned
        debug_assert!(
            addr == (addr & 0xffff_fffe),
            " addr={:#x?}  {:#x?}",
            addr,
            (addr & 0xffff_fffe)
        );
        let b0 = self.read8(addr).unwrap() as u32;
        let b1 = self.read8(addr + 1).unwrap() as u32;
        let b2 = self.read8(addr + 2).unwrap() as u32;
        let b3 = self.read8(addr + 3).unwrap() as u32;
        return Some(b3 << 24 | b2 << 16 | b1 << 8 | b0);
    }

    fn write_32(&mut self, addr: VAddr, value: u32) {
        debug_assert!(addr == addr & !0x3);
        self.write32(addr, value);
        // self.write8(addr, (value & 0xff) as u8);
        // self.write8(addr + 1, ((value >> 8) & 0xff) as u8);
        // self.write8(addr + 2, ((value >> 16) & 0xff) as u8);
        // self.write8(addr + 3, ((value >> 24) & 0xff) as u8);
    }
}

impl MemoryOperations<RAM, u8> for RAM {
    fn write8(&mut self, addr: VAddr, value: u8) -> bool {
        // debug_assert!(
        //     addr >= self.base_address,
        //     "addr {:#x?} < {:#x?}",
        //     addr,
        //     self.base_address
        // );

        let offs = addr - self.base_address;
        // debug_assert!(
        //     offs < self.size as u64,
        //     "addr={:#x?}  base={:#x?} offs={:#x?}",
        //     addr,
        //     self.base_address,
        //     offs
        // );
        unsafe { ptr::write(self.data.offset(offs as isize), (value & 0xff) as u8) }
        true
    }

    // @TODO: Will panic if reading out of bounds
    fn read8(&self, addr: VAddr) -> Option<u8> {
        debug_assert!(addr >= self.base_address);
        debug_assert!(addr < self.size.checked_add(self.base_address as usize).unwrap() as u64);
        let offs = addr - self.base_address;

        unsafe { Some(ptr::read(self.data.offset(offs as isize)) as u8) }
    }

    fn read32(&self, addr: VAddr) -> Option<u32> {
        let ptr = self.data as *mut u32;
        let offs = (addr - self.base_address) >> 2;

        unsafe { Some(ptr::read(ptr.offset(offs as isize))) }
    }

    fn write32(&mut self, addr: VAddr, value: u32) {
        let ptr = self.data as *mut u32;
        let offs = (addr - self.base_address) >> 2;

        unsafe { ptr::write(ptr.offset(offs as isize), value) }
    }
}

impl RAM {
    pub fn create(base_address: u64, size: usize) -> RAM {
        let data = Vec::<u32>::with_capacity(size >> 2).as_mut_ptr();
        mem::forget(data);
        for i in (0..size >> 2).step_by(4) {
            unsafe {
                data.offset(i as isize).write(0);
            }
        }
        RAM {
            base_address,
            size,
            data: data as *mut u8,
        }
    }
}

// impl Memory {
//     pub fn create() -> impl MemoryOperations<Memory> {
//         Memory {
//             segments: Vec::new(),
//         }
//     }

//     fn find_segment(&self, addr: VAddr) -> Option<MemorySegment> {
//         match self.segments.iter().find(|&s| {
//             s.base_address <= addr && s.base_address.saturating_add(s.size as u64) > addr
//         }) {
//             Some(segment) => Some(*segment),
//             None => {
//                 panic!("Missing memory segment for {:#x?}", addr);
//                 None
//             }
//         }
//     }

//     // pub fn dump(&self, addr: usize, count: usize) {
//     //     for i in 0..count as isize {
//     //         unsafe {
//     //             print!(
//     //                 "{:#x?} ",
//     //                 self.data
//     //                     .offset(i + (addr - self.base_address as usize) as isize)
//     //                     .read()
//     //             )
//     //         };
//     //     }
//     // }
// }
