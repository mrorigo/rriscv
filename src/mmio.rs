use elfloader::VAddr;

use crate::{
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

pub struct PLIC {
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
        let h = self
            .read32(CLINT::CLINT_MTIME as VAddr + 4 + self.range.start)
            .unwrap();
        let l = self
            .read32(CLINT::CLINT_MTIME as VAddr + 0 + self.range.start)
            .unwrap();

        if l.wrapping_add(1) < l {
            self.write32(
                CLINT::CLINT_MTIME as VAddr + 4 + self.range.start,
                h.wrapping_add(1),
            );
        }
        if l % 1000000 == 0 {
            println!("CLINT:MTIME: {:#?} / {:#?}", l, h);
        }

        self.write32(
            CLINT::CLINT_MTIME as VAddr + 0 + self.range.start,
            l.wrapping_add(1),
        );

        // Update CLINT_MTIME
    }
}

impl PLIC {
    pub fn create(range: MemoryRange) -> PLIC {
        let ram = RAM::create(range.start, (range.end - range.start).try_into().unwrap());
        PLIC { range, ram }
    }
}

pub trait VirtualDevice {
    fn name(&self) -> &str;
    fn includes(&self, addr: VAddr) -> bool;
    fn write(&mut self, addr: VAddr, value: u8) -> bool;
    fn read(&self, addr: VAddr) -> u8;
    fn tick(&mut self) {}
}

impl MemoryOperations<PhysicalMemory, u8> for PhysicalMemory {
    fn read8(&self, addr: VAddr) -> Option<u8> {
        self.ram.read8(addr)
    }

    fn write8(&mut self, addr: VAddr, value: u8) -> bool {
        self.ram.write8(addr, value)
    }

    fn read32(&self, addr: VAddr) -> Option<u32> {
        self.ram.read32(addr)
    }

    fn write32(&mut self, addr: VAddr, value: u32) {
        self.ram.write32(addr, value)
    }

    fn read64(&self, addr: VAddr) -> Option<u64> {
        self.ram.read64(addr)
    }

    fn write64(&mut self, addr: VAddr, value: u64) {
        self.ram.write64(addr, value)
    }

    fn read16(&self, addr: VAddr) -> Option<u16> {
        self.ram.read16(addr)
    }

    fn write16(&mut self, addr: VAddr, value: u16) {
        self.ram.write16(addr, value);
    }
}

impl VirtualDevice for PhysicalMemory {
    fn name(&self) -> &str {
        "memory"
    }
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn write(&mut self, _addr: VAddr, _value: u8) -> bool {
        panic!("")
    }

    fn read(&self, _addr: VAddr) -> u8 {
        todo!()
    }
}

impl VirtualDevice for PLIC {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }

    fn write(&mut self, addr: VAddr, value: u8) -> bool {
        self.ram.write8(addr, value)
    }

    fn read(&self, addr: VAddr) -> u8 {
        self.ram.read8(addr).unwrap()
    }
}

impl MemoryOperations<PLIC, u8> for PLIC {
    fn read8(&self, addr: VAddr) -> Option<u8> {
        self.ram.read8(addr)
    }

    fn write8(&mut self, addr: VAddr, value: u8) -> bool {
        self.ram.write8(addr, value)
    }

    fn read16(&self, addr: VAddr) -> Option<u16> {
        self.ram.read16(addr)
    }

    fn write16(&mut self, addr: VAddr, value: u16) {
        self.ram.write16(addr, value)
    }

    fn read32(&self, addr: VAddr) -> Option<u32> {
        self.ram.read32(addr)
    }
    fn write32(&mut self, addr: VAddr, value: u32) {
        self.ram.write32(addr, value)
    }

    fn read64(&self, addr: VAddr) -> Option<u64> {
        self.ram.read64(addr)
    }
    fn write64(&mut self, addr: VAddr, value: u64) {
        self.ram.write64(addr, value)
    }
}

impl MemoryOperations<CLINT, u8> for CLINT {
    fn read8(&self, addr: VAddr) -> Option<u8> {
        self.ram.read8(addr)
    }

    fn write8(&mut self, addr: VAddr, value: u8) -> bool {
        self.ram.write8(addr, value)
    }

    fn read32(&self, addr: VAddr) -> Option<u32> {
        self.ram.read32(addr)
    }

    fn write32(&mut self, addr: VAddr, value: u32) {
        self.ram.write32(addr, value)
    }

    fn read64(&self, addr: VAddr) -> Option<u64> {
        todo!()
    }

    fn write64(&mut self, addr: VAddr, value: u64) {
        todo!()
    }

    fn read16(&self, addr: VAddr) -> Option<u16> {
        todo!()
    }

    fn write16(&mut self, addr: VAddr, value: u16) {
        todo!()
    }
}

impl VirtualDevice for CLINT {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }

    fn write(&mut self, addr: VAddr, value: u8) -> bool {
        self.ram.write8(addr, value)
    }

    fn read(&self, addr: VAddr) -> u8 {
        self.ram.read8(addr).unwrap()
    }
}
