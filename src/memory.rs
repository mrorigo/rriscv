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
pub trait MemoryOperations<T, T2: MemoryCellType> {
    fn read8(&self, addr: VAddr) -> Option<T2>;
    fn write8(&mut self, addr: VAddr, value: T2) -> bool;

    fn read64(&self, addr: VAddr) -> Option<u64>;
    fn write64(&mut self, addr: VAddr, value: u64);

    fn read32(&self, addr: VAddr) -> Option<u32>;
    fn write32(&mut self, addr: VAddr, value: u32);

    fn read16(&self, addr: VAddr) -> Option<u16>;
    fn write16(&mut self, addr: VAddr, value: u16);

    //    fn add_segment(&mut self, base_address: VAddr, size: usize);
}

pub trait RAMOperations<T>: MemoryOperations<T, u8> {
    fn read_32(&self, addr: VAddr) -> Option<u32> {
        // @FIXME: We allow 16-bit aligned access, because instruction fetches are 16-bit aligned
        // debug_assert!(
        //     addr == (addr & 0xffff_fffe),
        //     " addr={:#x?}  {:#x?}",
        //     addr,
        //     (addr & 0xffff_fffe)
        // );
        let val = self.read32(addr).unwrap();
        return Some(val);
    }

    fn write_32(&mut self, addr: VAddr, value: u32) {
        //        debug_assert!(addr == addr & !0x3);
        self.write32(addr, value);
    }
}

impl MemoryOperations<RAM, u8> for RAM {
    fn write8(&mut self, addr: VAddr, value: u8) -> bool {
        debug_assert!(addr >= self.base_address);
        debug_assert!(addr < self.size.checked_add(self.base_address as usize).unwrap() as u64);

        let offs = addr - self.base_address;
        // if addr >= 0x80000000 && addr < 0x80002200 {
        //     println!("w8 {:#x?} @ {:#x?}", value, addr);
        // }
        unsafe { ptr::write(self.data.offset(offs as isize), (value & 0xff) as u8) }
        true
    }

    // @TODO: Will panic if reading out of bounds
    fn read8(&self, addr: VAddr) -> Option<u8> {
        debug_assert!(addr >= self.base_address);
        debug_assert!(addr < self.size.checked_add(self.base_address as usize).unwrap() as u64);
        let offs = addr - self.base_address;

        let value = unsafe { ptr::read(self.data.offset(offs as isize)) as u8 };

        // if addr <= 0x80002228 && addr > 0x80000000 {
        //     println!("r8 {:#x?} @ {:#x?}", value, addr);
        // }

        Some(value)
    }

    fn read32(&self, addr: VAddr) -> Option<u32> {
        let offs = addr - self.base_address;

        let mut data = 0 as u32;
        for i in 0..4 {
            let b = unsafe {
                Some(ptr::read(self.data.offset(i + offs as isize)) as u8).unwrap() as u32
            };
            data |= b << (i * 8)
        }
        // if addr <= 0x80002228 && addr > 0x80002200 {
        //     println!("r32 {:#x?} => {:#x?}", addr, data);
        // }
        Some(data)
    }

    fn write32(&mut self, addr: VAddr, value: u32) {
        let mut v = value;
        for i in 0..4 {
            self.write8(addr + i, v as u8);
            v >>= 8;
        }
    }

    fn write16(&mut self, addr: VAddr, value: u16) {
        let mut v = value;
        for i in 0..2 {
            self.write8(addr + i, v as u8);
            v >>= 8;
        }
    }

    fn read16(&self, addr: VAddr) -> Option<u16> {
        todo!()
    }

    fn read64(&self, addr: VAddr) -> Option<u64> {
        todo!()
    }

    fn write64(&mut self, addr: VAddr, value: u64) {
        todo!()
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
        // for i in (0..size >> 2).step_by(4) {
        //     unsafe {
        //         assert!(data.offset(i as isize).read() == 0);
        //     }
        // }
        RAM {
            base_address,
            size,
            data: data as *mut u8,
        }
    }
}
