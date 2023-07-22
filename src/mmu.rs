use elfloader::VAddr;
use include_bytes_aligned::include_bytes_aligned;

use crate::{
    memory::{MemoryOperations, RAMOperations},
    mmio::{PhysicalMemory, VirtualDevice, CLINT, PLIC, UART, VIRTIO},
};

#[derive(Copy, Clone, Debug)]
pub struct MemoryRange {
    pub name: &'static str,
    pub start: VAddr,
    pub end: VAddr,
}

impl MemoryRange {
    pub fn includes(&self, addr: VAddr) -> bool {
        self.start <= addr && addr <= self.end
    }

    pub fn find_named_range(device_table: &Vec<MemoryRange>, name: &str) -> Option<MemoryRange> {
        let mut val = None;
        for i in 0..device_table.len() {
            if device_table[i].name == name {
                val = Some(device_table[i]);
                break;
            }
        }
        val
    }
}

#[derive(Debug)]
pub struct MMU {
    memory: PhysicalMemory,
    uart: UART,
    plic: PLIC,
    clint: CLINT,
    virtio: VIRTIO,

    device_table: Vec<MemoryRange>,
}

impl MemoryOperations<MMU, u8> for MMU {
    // @TODO: Optimize this?
    fn read_single(&self, addr: VAddr) -> Option<u8> {
        if self.memory.includes(addr) {
            self.memory.read_single(addr)
        } else if self.uart.includes(addr) {
            Some(self.uart.read(addr))
        } else if self.clint.includes(addr) {
            self.clint.read_single(addr)
        } else if self.plic.includes(addr) {
            todo!("PLIC I/O")
        } else if self.virtio.includes(addr) {
            todo!("VIRTIO I/O")
        } else {
            panic!("{:#x?} is not mapped to memory", addr)
        }
    }

    // @TODO: Optimize this?
    fn write_single(&mut self, addr: VAddr, value: u8) -> bool {
        if self.memory.includes(addr) {
            self.memory.write_single(addr, value)
        } else if self.uart.includes(addr) {
            self.uart.write(addr, value)
        } else if self.clint.includes(addr) {
            self.clint.write_single(addr, value)
        } else if self.plic.includes(addr) {
            todo!("PLIC I/O")
        } else if self.virtio.includes(addr) {
            todo!("VIRTIO I/O")
        } else {
            panic!("{:#x?} is not mapped to memory", addr)
        }
    }
}

impl RAMOperations<MMU> for MMU {}

impl MMU {
    pub fn create() -> MMU {
        let device_table = MMU::parse_dtb();
        let memory =
            PhysicalMemory::create(MemoryRange::find_named_range(&device_table, "memory").unwrap());
        let uart = UART::create(MemoryRange::find_named_range(&device_table, "uart").unwrap());
        let plic = PLIC::create(
            MemoryRange::find_named_range(&device_table, "interrupt-controller").unwrap(),
        );
        let clint = CLINT::create(MemoryRange::find_named_range(&device_table, "clint").unwrap());
        let virtio =
            VIRTIO::create(MemoryRange::find_named_range(&device_table, "virtio_mmio").unwrap());

        MMU {
            memory,
            uart,
            plic,
            clint,
            virtio,
            device_table,
        }
    }

    pub fn tick(&mut self) {
        self.uart.tick();
    }

    fn parse_dtb() -> Vec<MemoryRange> {
        let mut devs = Vec::<MemoryRange>::new();
        let content: &'static [u8; 1590] = (include_bytes_aligned!(64, "../resources/dtb.dtb"))
            .try_into()
            .unwrap();
        let mut curr_range: Option<MemoryRange> = None;
        let mut size = 0 as u32;
        match dtb::Reader::read(content) {
            Ok(dtb) => {
                dtb.struct_items().for_each(|rme| {
                    let node_name = match rme.node_name() {
                        Ok(node_name) => Some(node_name),
                        _ => Some(""),
                    };
                    let name = match rme.name() {
                        Ok(name) => Some(name),
                        _ => Some(""),
                    };

                    if rme.is_begin_node() && node_name.is_some() && name.unwrap().contains(&"@") {
                        match curr_range {
                            Some(range) => devs.push(MemoryRange {
                                name: range.name,
                                start: range.start,
                                end: range.start.saturating_add(size as u64),
                            }),
                            None => {}
                        }

                        let addr = u64::from_str_radix(rme.unit_address().unwrap(), 16)
                            .expect("Not a hex address");

                        if addr > 0x1000 {
                            curr_range = Some(MemoryRange {
                                name: node_name.unwrap(),
                                start: addr,
                                end: addr,
                            })
                        } else {
                            curr_range = None
                        }
                    } else if name.is_some()
                        && name.unwrap().contains("reg")
                        && curr_range.is_some()
                    {
                        let val = rme.value().expect("No reg value!?");
                        size = u32::from_be_bytes(val[12..16].try_into().unwrap());
                    }
                });

                match curr_range {
                    Some(range) => devs.push(MemoryRange {
                        name: range.name,
                        start: range.start,
                        end: range.start.saturating_add(size as u64),
                    }),
                    None => {}
                }
            }
            Err(err) => panic!("{:?}", err),
        }
        devs
    }

    pub fn dump_device_table(&self) {
        for i in 0..self.device_table.len() {
            println!("{:x?}", self.device_table[i]);
        }
    }
}

pub trait SV39Addr {
    fn level0(&self) -> u16; // 9 bits
    fn level1(&self) -> u16; // 9 bits
    fn level2(&self) -> u16; // 9 bits
    fn offset(&self) -> u16; // 12 bits
    fn tag(&self) -> u32; // 25 bits
}

impl SV39Addr for VAddr {
    fn tag(&self) -> u32 {
        return ((self >> 39) & 0x1ffffff) as u32;
    }
    fn offset(&self) -> u16 {
        return (self & 0xfff) as u16;
    }
    fn level0(&self) -> u16 {
        return ((self >> 12) & 0x1ff) as u16;
    }
    fn level1(&self) -> u16 {
        return ((self >> 21) & 0x1ff) as u16;
    }
    fn level2(&self) -> u16 {
        return ((self >> 30) & 0x1ff) as u16;
    }
}

#[repr(u8)]
pub enum PTEPermBit {
    VALID = 0,
    READ = 1,
    WRITE = 2,
    EXECUTE = 3,
    USER = 4,
}

type PageTableEntry = u64;

pub trait PTE {
    fn page_number(&self) -> u64; // 44 bits
    fn has_permission_bit(&self, bit: PTEPermBit) -> bool;
    fn set_permission_bit(&mut self, bit: PTEPermBit);
    fn clear_permission_bit(&mut self, bit: PTEPermBit);
}

impl PTE for PageTableEntry {
    fn page_number(&self) -> u64 {
        (self >> 10) & 0xfffffffffff
    }

    fn has_permission_bit(&self, bit: PTEPermBit) -> bool {
        ((self & 0x1f) & (1 << (bit as u8))) != 0
    }

    fn set_permission_bit(&mut self, bit: PTEPermBit) {
        *self |= 1 << (bit as u64);
    }

    fn clear_permission_bit(&mut self, bit: PTEPermBit) {
        *self ^= !(1 << (bit as u64) - 1);
    }
}
