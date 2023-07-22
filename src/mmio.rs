use elfloader::VAddr;

use crate::{
    cpu::TrapCause,
    memory::{MemoryOperations, RAM},
    mmu::MemoryRange,
};

// macro_rules! mmio_trace {
//     ($instr:expr) => {
//         print!("MMIO: ");
//         $instr;
//     };
// }

pub struct MMIODevice {}

pub struct CLINT {
    range: MemoryRange,
    ram: RAM,
}

pub struct PhysicalMemory {
    pub range: MemoryRange,
    ram: RAM,
}

impl PhysicalMemory {
    pub fn create(range: MemoryRange) -> PhysicalMemory {
        let ram = RAM::create(range.start, (range.end - range.start).try_into().unwrap());
        PhysicalMemory { range, ram }
    }
}

impl CLINT {
    pub const CLINT_MTIME: usize = 0xbff8;

    pub fn create(range: MemoryRange) -> CLINT {
        let ram = RAM::create(range.start, (range.end - range.start).try_into().unwrap());
        CLINT { range, ram }
    }

    pub fn tick(&mut self) {
        let l = self
            .read32(CLINT::CLINT_MTIME as VAddr + 0 + self.range.start)
            .unwrap();

        if l.wrapping_add(1) < l {
            let h = self
                .read32(CLINT::CLINT_MTIME as VAddr + 4 + self.range.start)
                .unwrap();
            self.write32(
                CLINT::CLINT_MTIME as VAddr + 4 + self.range.start,
                h.wrapping_add(1),
            );
        }
        if l % 10000000 == 0 {
            println!("CLINT:MTIME: {:#?}", l);
        }

        self.write32(
            CLINT::CLINT_MTIME as VAddr + 0 + self.range.start,
            l.wrapping_add(1),
        );

        // Update CLINT_MTIME
    }
}

pub trait VirtualDevice {
    fn name(&self) -> &str;
    fn includes(&self, addr: VAddr) -> bool;
    fn write(&mut self, addr: VAddr, value: u8) -> Option<TrapCause>;
    fn read(&mut self, addr: VAddr) -> Result<u8, TrapCause>;
    fn tick(&mut self) {}
}

impl MemoryOperations<PhysicalMemory, u8> for PhysicalMemory {
    fn read8(&mut self, addr: VAddr) -> Result<u8, TrapCause> {
        self.ram.read8(addr)
    }

    fn write8(&mut self, addr: VAddr, value: u8) -> Option<TrapCause> {
        self.ram.write8(addr, value)
    }

    fn read32(&mut self, addr: VAddr) -> Result<u32, TrapCause> {
        self.ram.read32(addr)
    }

    fn write32(&mut self, addr: VAddr, value: u32) -> Option<TrapCause> {
        self.ram.write32(addr, value)
    }

    fn read64(&mut self, addr: VAddr) -> Result<u64, TrapCause> {
        self.ram.read64(addr)
    }

    fn write64(&mut self, addr: VAddr, value: u64) -> Option<TrapCause> {
        self.ram.write64(addr, value)
    }

    fn read16(&mut self, addr: VAddr) -> Option<u16> {
        self.ram.read16(addr)
    }

    fn write16(&mut self, addr: VAddr, value: u16) -> Option<TrapCause> {
        self.ram.write16(addr, value)
    }
}

impl VirtualDevice for PhysicalMemory {
    fn name(&self) -> &str {
        "memory"
    }
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn write(&mut self, _addr: VAddr, _value: u8) -> Option<TrapCause> {
        panic!("")
    }

    fn read(&mut self, _addr: VAddr) -> Result<u8, TrapCause> {
        todo!()
    }
}

impl MemoryOperations<CLINT, u8> for CLINT {
    fn read8(&mut self, addr: VAddr) -> Result<u8, TrapCause> {
        self.ram.read8(addr)
    }

    fn write8(&mut self, addr: VAddr, value: u8) -> Option<TrapCause> {
        self.ram.write8(addr, value)
    }

    fn read32(&mut self, addr: VAddr) -> Result<u32, TrapCause> {
        self.ram.read32(addr)
    }

    fn write32(&mut self, addr: VAddr, value: u32) -> Option<TrapCause> {
        self.ram.write32(addr, value)
    }
}

impl VirtualDevice for CLINT {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }

    fn write(&mut self, addr: VAddr, value: u8) -> Option<TrapCause> {
        self.ram.write8(addr, value)
    }

    fn read(&mut self, addr: VAddr) -> Result<u8, TrapCause> {
        Ok(self.ram.read8(addr).unwrap())
    }
}
