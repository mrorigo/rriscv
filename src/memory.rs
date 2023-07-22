use std::mem;
use std::ptr;

use elfloader::VAddr;

// type VAddr = u64;
// type PAddr = u64;

#[derive(Debug, Copy, Clone)]
pub enum MemoryAccessWidth {
    BYTE,     // 8 bits
    HALFWORD, // 16 bits
    WORD,     // 32 bits
    LONG,
}

#[derive(Debug, Copy, Clone)]
pub struct MemorySegment {
    pub base_address: VAddr,
    pub size: usize,
    data: *mut u8,
}

#[derive(Debug)]
pub struct Memory {
    pub segments: Vec<MemorySegment>,
}

#[allow(unused_variables)]
pub trait MemoryOperations<T>: std::fmt::Debug {
    fn read_single(&self, addr: VAddr, memory_access_width: MemoryAccessWidth) -> Option<u64>;
    fn write_single(
        &mut self,
        addr: VAddr,
        value: u64,
        memory_access_width: MemoryAccessWidth,
    ) -> bool;
    fn add_segment(&mut self, base_address: VAddr, size: usize);
}

impl MemoryOperations<Memory> for Memory {
    fn write_single(&mut self, addr: VAddr, value: u64, maw: MemoryAccessWidth) -> bool {
        let mut segment = self.find_segment(addr).unwrap();
        segment.write_single(addr, value, maw)
    }
    fn read_single(&self, addr: VAddr, maw: MemoryAccessWidth) -> Option<u64> {
        self.find_segment(addr).unwrap().read_single(addr, maw)
    }

    fn add_segment(&mut self, base_address: VAddr, size: usize) {
        let segment = MemorySegment::create(base_address, size);
        self.segments.push(segment);
    }
}

impl MemorySegment {
    fn write_single(&mut self, addr: VAddr, value: u64, maw: MemoryAccessWidth) -> bool {
        //        println!("write_single @ {:#x} base={:#x}", addr, self.base_address);
        assert!(
            addr >= self.base_address,
            "addr {:#x?} < {:#x?}",
            addr,
            self.base_address
        );

        let offs = addr - self.base_address;
        assert!(
            offs < self.size as u64,
            "addr={:#x?}  base={:#x?} offs={:#x?}",
            addr,
            self.base_address,
            offs
        );
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
    fn read_single(&self, addr: VAddr, maw: MemoryAccessWidth) -> Option<u64> {
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
                (self
                    .read_single(addr + 2, MemoryAccessWidth::HALFWORD)
                    .unwrap()
                    << 16)
                    | self.read_single(addr, MemoryAccessWidth::HALFWORD).unwrap() & 0xffff,
            ),
            MemoryAccessWidth::LONG => Some(
                ((self.read_single(addr + 4, MemoryAccessWidth::WORD).unwrap() << 32)
                    | self.read_single(addr, MemoryAccessWidth::WORD).unwrap())
                    & 0xffffffff,
            ),
        }
    }
}

impl MemorySegment {
    pub fn create(base_address: u64, size: usize) -> MemorySegment {
        let data = Vec::<u8>::with_capacity(size).as_mut_ptr();
        mem::forget(data);
        for i in 0..size {
            unsafe {
                data.offset(i as isize).write(0);
            }
        }
        MemorySegment {
            base_address,
            size,
            data: data,
        }
    }
}

impl Memory {
    pub fn create() -> impl MemoryOperations<Memory> {
        Memory {
            segments: Vec::new(),
        }
    }

    fn find_segment(&self, addr: VAddr) -> Option<MemorySegment> {
        match self.segments.iter().find(|&s| {
            s.base_address <= addr && s.base_address.saturating_add(s.size as u64) > addr
        }) {
            Some(segment) => Some(*segment),
            None => {
                panic!("Missing memory segment for {:#x?}", addr);
                None
            }
        }
    }

    // pub fn dump(&self, addr: usize, count: usize) {
    //     for i in 0..count as isize {
    //         unsafe {
    //             print!(
    //                 "{:#x?} ",
    //                 self.data
    //                     .offset(i + (addr - self.base_address as usize) as isize)
    //                     .read()
    //             )
    //         };
    //     }
    // }
}
