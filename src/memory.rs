use std::mem;
use std::ptr;

#[repr(u8)]
pub enum Permission {
    READ = 1,
    WRITE = 2,
    RW = 3,
}

#[derive(Debug, Copy, Clone)]
pub enum MemoryAccessWidth {
    BYTE,     // 8 bits
    HALFWORD, // 16 bits
    WORD,     // 32 bits
    LONG,
}
// pub const BITS8 : MemoryAccessWidth = MemoryAccessWidth::BYTE;
// pub const BITS16 : MemoryAccessWidth = MemoryAccessWidth::HALFWORD;
// pub const BITS32 : MemoryAccessWidth = MemoryAccessWidth::WORD;
// pub const BITS64 : MemoryAccessWidth = MemoryAccessWidth::LONG;

#[derive(Debug)]
pub struct Memory {
    pub base_address: u64,
    pub size: usize,
    data: *mut u8,
}

#[allow(unused_variables)]
pub trait MemoryOperations: std::fmt::Debug {
    fn get_base_address(&self) -> u64;
    fn read_single(&self, addr: u64, memory_access_width: MemoryAccessWidth) -> Option<u64> {
        None
    }
    fn write_single(
        &mut self,
        addr: u64,
        value: u64,
        memory_access_width: MemoryAccessWidth,
    ) -> bool {
        false
    }
}

impl MemoryOperations for Memory {
    fn write_single(&mut self, addr: u64, value: u64, maw: MemoryAccessWidth) -> bool {
        //        println!("write_single @ {:#x} base={:#x}", addr, self.base_address);
        assert!(
            addr >= self.base_address,
            "addr {:#x?} < {:#x?}",
            addr,
            self.base_address
        );
        assert!(
            addr < self.size.checked_add(self.base_address as usize).unwrap() as u64,
            "addr {:#x?} >= {:#x?} ",
            addr,
            self.size.checked_add(self.base_address as usize).unwrap()
        );
        let offs = addr - self.base_address;
        match maw {
            MemoryAccessWidth::BYTE => unsafe {
                ptr::write(self.data.offset(offs as isize), (value & 0xff) as u8);
            },
            MemoryAccessWidth::HALFWORD => {
                self.write_single(addr, value >> 8, MemoryAccessWidth::BYTE);
                self.write_single(addr + 1, value, MemoryAccessWidth::BYTE);
            }
            MemoryAccessWidth::WORD => {
                self.write_single(addr, value >> 16, MemoryAccessWidth::HALFWORD);
                self.write_single(addr + 2, value, MemoryAccessWidth::HALFWORD);
            }
            MemoryAccessWidth::LONG => {
                self.write_single(addr, value >> 32, MemoryAccessWidth::WORD);
                self.write_single(addr + 4, value, MemoryAccessWidth::WORD);
            }
        }
        true
    }

    // @TODO: Will panic if reading out of bounds
    fn read_single(&self, addr: u64, maw: MemoryAccessWidth) -> Option<u64> {
        // println!(
        //     "read_single {:?} @ {:#x} base={:#x}",
        //     maw, addr, self.base_address
        // );
        //00009117
        //17910000
        assert!(addr >= self.base_address);
        assert!(addr < self.size.checked_add(self.base_address as usize).unwrap() as u64);
        let offs = addr - self.base_address;
        match maw {
            MemoryAccessWidth::BYTE => unsafe {
                Some(ptr::read(self.data.offset(offs as isize)) as u64 & 0xff)
            },
            MemoryAccessWidth::HALFWORD => Some(
                (self.read_single(addr + 1, MemoryAccessWidth::BYTE).unwrap() << 8
                    | (self.read_single(addr, MemoryAccessWidth::BYTE).unwrap()))
                    as u64
                    & 0xffff,
            ),
            MemoryAccessWidth::WORD => Some(
                ((self
                    .read_single(addr + 2, MemoryAccessWidth::HALFWORD)
                    .unwrap()
                    << 16)
                    | self.read_single(addr, MemoryAccessWidth::HALFWORD).unwrap() & 0xffff),
            ),
            MemoryAccessWidth::LONG => Some(
                ((self.read_single(addr + 4, MemoryAccessWidth::WORD).unwrap() << 32)
                    | self.read_single(addr, MemoryAccessWidth::WORD).unwrap())
                    & 0xffffffff,
            ),
        }
    }

    fn get_base_address(&self) -> u64 {
        self.base_address
    }
}

impl Memory {
    pub fn create(base_address: u64, size: usize) -> impl MemoryOperations {
        let mut data = Vec::<u8>::with_capacity(size).as_mut_ptr();
        mem::forget(data);
        for i in 0..size {
            unsafe {
                data.offset(i as isize).write(0);
            }
        }
        Memory {
            base_address,
            size,
            data: data,
        }
    }

    pub fn dump(&self, addr: usize, count: usize) {
        for i in 0..count as isize {
            unsafe {
                print!(
                    "{:#x?} ",
                    self.data
                        .offset(i + (addr - self.base_address as usize) as isize)
                        .read()
                )
            };
        }
    }
}
