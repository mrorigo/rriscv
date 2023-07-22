use std::mem;
use std::ptr;
use std::u8;

use elfloader::VAddr;

use crate::cpu::TrapCause;

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
    fn read8(&mut self, addr: VAddr) -> Result<T2, TrapCause> {
        todo!()
    }
    fn write8(&mut self, addr: VAddr, value: T2) -> Option<TrapCause> {
        todo!()
    }

    fn read64(&mut self, addr: VAddr) -> Result<u64, TrapCause> {
        todo!()
    }
    fn write64(&mut self, addr: VAddr, value: u64) -> Option<TrapCause> {
        todo!()
    }

    fn read32(&mut self, addr: VAddr) -> Result<u32, TrapCause> {
        todo!()
    }
    fn write32(&mut self, addr: VAddr, value: u32) -> Option<TrapCause>;

    fn read16(&mut self, addr: VAddr) -> Option<u16> {
        todo!()
    }
    fn write16(&mut self, addr: VAddr, value: u16) -> Option<TrapCause> {
        todo!()
    }

    //    fn add_segment(&mut self, base_address: VAddr, size: usize);
}

pub trait RAMOperations<T>: MemoryOperations<T, u8> {
    fn read_32(&mut self, addr: VAddr) -> Result<u32, TrapCause> {
        self.read32(addr)
    }

    fn write_32(&mut self, addr: VAddr, value: u32) {
        self.write32(addr, value);
    }
}

impl MemoryOperations<RAM, u8> for RAM {
    fn write8(&mut self, addr: VAddr, value: u8) -> Option<TrapCause> {
        if addr < self.base_address
            || addr >= self.size.checked_add(self.base_address as usize).unwrap() as u64
        {
            return Some(TrapCause::StoreAccessFault(addr));
        }

        let offs = addr - self.base_address;
        unsafe { ptr::write(self.data.offset(offs as isize), (value & 0xff) as u8) }
        None
    }

    // @TODO: Will panic if reading out of bounds
    fn read8(&mut self, addr: VAddr) -> Result<u8, TrapCause> {
        debug_assert!(addr >= self.base_address);
        debug_assert!(addr < self.size.checked_add(self.base_address as usize).unwrap() as u64);
        let offs = addr - self.base_address;

        let value = unsafe { ptr::read(self.data.offset(offs as isize)) as u8 };

        // if addr <= 0x80002228 && addr > 0x80000000 {
        //     println!("r8 {:#x?} @ {:#x?}", value, addr);
        // }

        Ok(value)
    }

    fn read32(&mut self, addr: VAddr) -> Result<u32, TrapCause> {
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
        Ok(data)
    }

    fn write32(&mut self, addr: VAddr, value: u32) -> Option<TrapCause> {
        let mut v = value;
        for i in 0..4 {
            match self.write8(addr + i, v as u8) {
                Some(trap) => return Some(trap),
                None => {}
            }
            v >>= 8;
        }
        None
    }

    fn write16(&mut self, addr: VAddr, value: u16) -> Option<TrapCause> {
        let mut v = value;
        for i in 0..2 {
            match self.write8(addr + i, v as u8) {
                Some(trap) => return Some(trap),
                None => {}
            }
            v >>= 8;
        }
        None
    }

    fn read16(&mut self, addr: VAddr) -> Option<u16> {
        assert!(addr > self.base_address, "Addr {:#x?}", addr);
        let offs = addr - self.base_address;

        let mut data = 0 as u16;
        for i in 0..2 {
            let b = unsafe {
                Some(ptr::read(self.data.offset(i + offs as isize)) as u8).unwrap() as u16
            };
            data |= b << (i * 8)
        }
        Some(data)
    }

    fn read64(&mut self, addr: VAddr) -> Result<u64, TrapCause> {
        let l = match self.read32(addr) {
            Err(cause) => return Err(cause),
            Ok(val) => val,
        };
        let h = match self.read32(addr + 4) {
            Err(cause) => return Err(cause),
            Ok(val) => val,
        };
        let comp = ((h as u64) << 32) | l as u64;
        Ok(comp)
    }

    fn write64(&mut self, addr: VAddr, value: u64) -> Option<TrapCause> {
        match self.write32(addr, value as u32) {
            None => {
                self.write32(addr + 4, (value >> 32) as u32);
                None
            }
            Some(cause) => Some(cause),
        }

        //        todo!()
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
