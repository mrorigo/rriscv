use elfloader::VAddr;

use crate::{
    memory::{self, MemoryOperations, RAM},
    mmu::MemoryRange,
};

// #[derive(Debug, Copy, Clone)]
// pub enum MMIODevice {
//     UART(MemoryRange),
//     MEMORY(MemoryRange),
//     CLINT(MemoryRange),
//     IC(MemoryRange),
//     VIRTIO(MemoryRange),
// }

pub struct MMIODevice {}

impl MMIODevice {
    pub fn create(range: MemoryRange) -> Box<dyn VirtualDevice> {
        match range.name {
            "memory" => Box::new(PhysicalMemory::create(range)),
            "uart" => Box::new(UART::create(range)),
            _ => panic!(),
        }
    }
}
#[derive(Debug)]
pub struct VIRTIO {
    range: MemoryRange,
}

#[derive(Debug)]
pub struct CLINT {
    range: MemoryRange,
    ram: RAM,
}

#[derive(Debug)]
pub struct PLIC {
    range: MemoryRange,
    ram: RAM,
}

#[derive(Debug)]
pub struct UART {
    range: MemoryRange,
}

#[derive(Debug)]
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

impl UART {
    pub fn create(range: MemoryRange) -> UART {
        UART { range }
    }
}

impl CLINT {
    pub fn create(range: MemoryRange) -> CLINT {
        let ram = RAM::create(range.start, (range.end - range.start).try_into().unwrap());
        CLINT { range, ram }
    }
}

impl PLIC {
    pub fn create(range: MemoryRange) -> PLIC {
        let ram = RAM::create(range.start, (range.end - range.start).try_into().unwrap());
        PLIC { range, ram }
    }
}

impl VIRTIO {
    pub fn create(range: MemoryRange) -> VIRTIO {
        VIRTIO { range }
    }
}

pub trait VirtualDevice: std::fmt::Debug {
    fn name(&self) -> &str;
    fn includes(&self, addr: VAddr) -> bool;
}

// impl MemoryOperations<PhysicalMemory> for PhysicalMemory {}
// impl MemoryOperations<UART> for UART {}

impl MemoryOperations<PhysicalMemory> for PhysicalMemory {
    fn read_single(
        &self,
        addr: VAddr,
        memory_access_width: memory::MemoryAccessWidth,
    ) -> Option<u64> {
        self.ram.read_single(addr, memory_access_width)
    }

    fn write_single(
        &mut self,
        addr: VAddr,
        value: u64,
        memory_access_width: memory::MemoryAccessWidth,
    ) -> bool {
        self.ram.write_single(addr, value, memory_access_width)
    }
}

impl VirtualDevice for PhysicalMemory {
    fn name(&self) -> &str {
        "memory"
    }
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }
}

impl VirtualDevice for UART {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }
}

impl VirtualDevice for PLIC {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }
}

impl MemoryOperations<CLINT> for CLINT {
    fn read_single(
        &self,
        addr: VAddr,
        memory_access_width: memory::MemoryAccessWidth,
    ) -> Option<u64> {
        self.ram.read_single(addr, memory_access_width)
    }

    fn write_single(
        &mut self,
        addr: VAddr,
        value: u64,
        memory_access_width: memory::MemoryAccessWidth,
    ) -> bool {
        self.ram.write_single(addr, value, memory_access_width)
    }
}

impl VirtualDevice for CLINT {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }
}

impl VirtualDevice for VIRTIO {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }
}

// impl MMIODevice {
//     pub fn create(range: MemoryRange) -> impl VirtualDevice<T> {
//         match range.name {
//             "memory" => MMIOD,
//             "uart" => UART::create(range),
//             // "clint" => MMIODevice::CLINT(range),
//             // "virtio_mmio" => MMIODevice::VIRTIO(range),
//             // "interrupt-controller" => MMIODevice::IC(range),
//             _ => panic!("Cannot create MMIO device of type {}", range.name),
//         }
//     }

//     // pub fn map(&self, addr: VAddr) -> impl VirtualDevice {
//     //     match self {
//     //         MMIODevice::UART(range) => UART::create(*range),
//     //         MMIODevice::MEMORY(range) => PhysicalMemory::create(*range),
//     //         MMIODevice::CLINT(_) => todo!(),
//     //         MMIODevice::IC(_) => todo!(),
//     //         MMIODevice::VIRTIO(_) => todo!(),
//     //     }
//     // }
// }
