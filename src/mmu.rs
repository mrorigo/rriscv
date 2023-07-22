use elfloader::{PAddr, VAddr};
use include_bytes_aligned::include_bytes_aligned;

use crate::{
    cpu::{PrivMode, RegisterValue, TrapCause, Xlen},
    memory::{MemoryOperations, RAMOperations},
    mmio::{PhysicalMemory, VirtualDevice, CLINT},
    plic::PLIC,
    uart::UART,
    virtio::VIRTIO,
};

#[derive(Copy, Clone)]
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

#[derive(PartialEq)]
pub enum AddressingMode {
    None = 0,
    SV32 = 1,
    SV39 = 2,
}

pub enum MemoryAccessType {
    READ = 1,
    WRITE = 2,
    EXECUTE = 3,
}

//#[derive(Debug)]
pub struct MMU {
    memory: PhysicalMemory,
    uart: UART,
    plic: PLIC,
    clint: CLINT,
    virtio: VIRTIO,
    //protected: Vec<MemoryRange>,
    device_table: Vec<MemoryRange>,
    pmode: PrivMode,
    mstatus: RegisterValue,
    satp: RegisterValue,
    ppn: u64,
    addressing_mode: AddressingMode,
}

impl MemoryOperations<MMU, u8> for MMU {
    // @TODO: Optimize this?
    fn read8(&mut self, addr: VAddr) -> Result<u8, TrapCause> {
        let resolved = self.translate_address(&addr, MemoryAccessType::READ);
        let addr = match resolved {
            None => return Err(TrapCause::LoadAccessFault(addr)),
            Some(val) => val,
        };

        if self.memory.includes(addr) {
            self.memory.read8(addr)
        } else if self.virtio.includes(addr) {
            todo!("VIRTIO I/O")
        } else if self.uart.includes(addr) {
            self.uart.read(addr)
        } else if self.clint.includes(addr) {
            self.clint.read8(addr)
        } else if self.plic.includes(addr) {
            todo!("PLIC I/O")
        } else {
            Err(TrapCause::Breakpoint)
            //panic!("{:#x?} is not mapped to memory", addr)
        }
    }

    // @TODO: Optimize this?
    fn write8(&mut self, addr: VAddr, value: u8) -> Option<TrapCause> {
        let resolved = self.translate_address(&addr, MemoryAccessType::WRITE);
        let addr = match resolved {
            None => return Some(TrapCause::StoreAccessFault),
            Some(val) => val,
        };
        // for i in 0..self.protected.len() {
        //     if self.protected[i].includes(addr) {
        //         panic!();
        //     }
        // }
        if self.memory.includes(addr) {
            self.memory.write8(addr, value)
        } else if self.uart.includes(addr) {
            self.uart.write(addr, value)
        } else if self.clint.includes(addr) {
            self.clint.write8(addr, value)
        } else if self.plic.includes(addr) {
            todo!("PLIC I/O")
        } else if self.virtio.includes(addr) {
            todo!("VIRTIO I/O")
        } else {
            panic!("{:#x?} is not mapped to memory", addr);
            Some(TrapCause::LoadAccessFault(addr))
        }
    }

    fn read32(&mut self, addr: VAddr) -> Result<u32, TrapCause> {
        let resolved = self.translate_address(&addr, MemoryAccessType::READ);
        let addr = match resolved {
            None => return Err(TrapCause::StoreAccessFault),
            Some(val) => val,
        };

        let value = if self.memory.includes(addr) {
            self.memory.read32(addr)
        } else if self.virtio.includes(addr) {
            self.virtio.read32(addr)
        } else if self.clint.includes(addr) {
            self.clint.read32(addr)
        } else if self.plic.includes(addr) {
            self.plic.read32(addr)
        } else {
            // panic!(
            //     "{:#x?} is not mapped to memory: {:#x?} - {:#x?}",
            //     addr, self.memory.range.start, self.memory.range.end
            // );
            return Err(TrapCause::StoreAccessFault);
        };
        Ok(value.unwrap())
    }

    fn write32(&mut self, addr: VAddr, value: u32) -> Option<TrapCause> {
        let resolved = self.translate_address(&addr, MemoryAccessType::WRITE);
        let addr = match resolved {
            None => return Some(TrapCause::StoreAccessFault),
            Some(val) => val,
        };
        if self.memory.includes(addr) {
            self.memory.write32(addr, value)
        } else if self.virtio.includes(addr) {
            self.virtio.write32(addr, value)
        } else if self.clint.includes(addr) {
            self.clint.write32(addr, value)
        } else if self.plic.includes(addr) {
            self.plic.write32(addr, value)
        } else {
            Some(TrapCause::LoadAccessFault(addr))
        }
    }

    fn read64(&mut self, addr: VAddr) -> Result<u64, TrapCause> {
        todo!()
    }

    fn write64(&mut self, _addr: VAddr, _value: u64) -> Option<TrapCause> {
        todo!()
    }

    fn read16(&mut self, _addr: VAddr) -> Option<u16> {
        todo!()
    }

    fn write16(&mut self, _addr: VAddr, _value: u16) -> Option<TrapCause> {
        todo!()
    }
}

impl RAMOperations<MMU> for MMU {}

impl MMU {
    // @TODO: Panics if devicetable is corrupt or not what we expect
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
            //protected: Vec::new(),
            device_table,
            pmode: PrivMode::Machine,
            mstatus: 0,
            satp: 0,
            ppn: 0,
            addressing_mode: AddressingMode::None,
        }
    }

    pub fn update_satp(&mut self, satp: RegisterValue, xlen: Xlen) {
        if self.satp == satp {
            return;
        }
        self.satp = satp;
        self.addressing_mode = match xlen {
            Xlen::Bits32 => match satp & 0x80000000 {
                0 => AddressingMode::None,
                _ => panic!("not implemented: SV32"),
            },
            Xlen::Bits64 => match satp >> 60 {
                0 => AddressingMode::None,
                8 => AddressingMode::SV39,
                9 => panic!("not implemented: SV48"),
                _ => {
                    println!("Unknown addressing_mode {:x}", satp >> 60);
                    panic!();
                }
            },
        };

        self.ppn = match xlen {
            Xlen::Bits32 => satp & 0x3fffff,
            Xlen::Bits64 => satp & 0xfffffffffff,
        };
    }

    fn traverse_pagetable(
        &mut self,
        addr: VAddr,
        level: usize,
        parent_ppn: u64,
        vpns: &[u64; 3],
        access_type: MemoryAccessType,
    ) -> Option<PAddr> {
        const PAGESIZE: u64 = 4096;
        let ptesize: u64 = match self.addressing_mode {
            AddressingMode::SV32 => 4,
            _ => 8,
        };
        let pte_addr = parent_ppn * PAGESIZE + vpns[level] * ptesize;
        let pte = match self.addressing_mode {
            AddressingMode::SV32 => self.memory.read32(pte_addr).unwrap() as PageTableEntry,
            _ => self.memory.read64(pte_addr).unwrap() as PageTableEntry,
        };
        let ppn = match self.addressing_mode {
            AddressingMode::SV32 => (pte >> 10) & 0x3fffff,
            _ => (pte >> 10) & 0xfffffffffff,
        };
        let a = pte.has_permission_bit(PTEPermBit::ACCESSED);
        let d = pte.has_permission_bit(PTEPermBit::DIRTY);
        let x = pte.has_permission_bit(PTEPermBit::EXECUTE);
        let v = pte.has_permission_bit(PTEPermBit::VALID);
        let w = pte.has_permission_bit(PTEPermBit::WRITE);
        let r = pte.has_permission_bit(PTEPermBit::READ);
        if v == false || (r == false && w == true) {
            return None;
        }

        if r == false && x == false {
            return match level {
                0 => return None,
                _ => self.traverse_pagetable(addr, level - 1, ppn, vpns, access_type),
            };
        }

        // Leaf page!

        if a == false
            || (match access_type {
                MemoryAccessType::WRITE => d == false,
                _ => false,
            })
        {
            let new_pte = pte
                | (1 << PTEPermBit::ACCESSED as u8)
                | (match access_type {
                    MemoryAccessType::WRITE => 1 << 7,
                    _ => 0,
                });
            match self.addressing_mode {
                AddressingMode::SV32 => self.memory.write32(pte_addr, new_pte as u32),
                _ => self.memory.write64(pte_addr, new_pte),
            };
        }

        match access_type {
            MemoryAccessType::EXECUTE => {
                if x == false {
                    return None;
                }
            }
            MemoryAccessType::READ => {
                if r == false {
                    return None;
                }
            }
            MemoryAccessType::WRITE => {
                if w == false {
                    return None;
                }
            }
            _ => {}
        };

        let ppns = match self.addressing_mode {
            AddressingMode::SV32 => [(pte >> 10) & 0x3ff, (pte >> 20) & 0xfff, 0 /*dummy*/],
            AddressingMode::SV39 => [
                (pte >> 10) & 0x1ff,
                (pte >> 19) & 0x1ff,
                (pte >> 28) & 0x3ffffff,
            ],
            _ => unreachable!(),
        };

        let p_address = match self.addressing_mode {
            AddressingMode::SV32 => match level {
                1 => {
                    if ppns[0] != 0 {
                        return None;
                    }
                    (ppns[1] << 22) | (vpns[0] << 12) | addr.offset()
                }
                0 => (ppn << 12) | addr.offset(),
                _ => unreachable!(),
            },
            _ => match level {
                2 => {
                    if ppns[1] != 0 || ppns[0] != 0 {
                        return None;
                    }
                    (ppns[2] << 30) | (vpns[1] << 21) | (vpns[0] << 12) | addr.offset()
                }
                1 => {
                    if ppns[0] != 0 {
                        return None;
                    }
                    (ppns[2] << 30) | (ppns[1] << 21) | (vpns[0] << 12) | addr.offset()
                }
                0 => (ppn << 12) | addr.offset(),
                _ => unreachable!(),
            },
        };

        //println!("PA:{:X}", p_address);
        Some(p_address)
    }

    pub fn translate_address(
        &mut self,
        va: &dyn SV39Addr,
        access_type: MemoryAccessType,
    ) -> Option<PAddr> {
        match self.pmode {
            PrivMode::Machine => Some(va.address() as PAddr),
            PrivMode::Supervisor | PrivMode::User => match self.addressing_mode {
                AddressingMode::None => Some(va.address()),
                AddressingMode::SV32 => todo!(),
                AddressingMode::SV39 => self.traverse_pagetable(
                    va.address(),
                    3 - 1,
                    self.ppn,
                    &va.get_vpns(),
                    access_type,
                ),
            },
            _ => panic!(),
        }
    }

    pub fn update_mstatus(&mut self, mstatus: RegisterValue) {
        self.mstatus = mstatus;
    }

    pub fn update_privilege_mode(&mut self, pmode: PrivMode) {
        if self.pmode != pmode {
            self.pmode = pmode;
            println!("MMU: New pmode: {:?}", pmode);
        }
    }

    pub fn virtio_mut(&mut self) -> &mut VIRTIO {
        return &mut self.virtio;
    }

    /// Returns new `mip` register value
    pub fn tick(&mut self, mip: RegisterValue) -> RegisterValue {
        self.clint.tick();
        self.virtio.tick();
        self.uart.tick();
        self.plic.tick(self.virtio.is_interrupting(), false, mip)
    }

    pub fn fetch(&mut self, addr: VAddr) -> Result<u32, TrapCause> {
        let addr = self.translate_address(&addr, MemoryAccessType::EXECUTE);

        match addr {
            Some(addr) => {
                if self.memory.includes(addr) {
                    match self.memory.read32(addr) {
                        Ok(value) => Ok(value),
                        Err(cause) => Err(TrapCause::InstructionAccessFault),
                    }
                } else {
                    Err(TrapCause::InstructionPageFault)
                }
            }
            None => Err(TrapCause::InstructionPageFault),
        }
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
            println!(
                "{:?} - {:#x?}-{:#x?}",
                self.device_table[i].name, self.device_table[i].start, self.device_table[i].end
            );
        }
    }
}

pub trait SV39Addr {
    fn level0(&self) -> u64; // 9 bits
    fn level1(&self) -> u64; // 9 bits
    fn level2(&self) -> u64; // 9 bits
    fn offset(&self) -> u64; // 12 bits
    fn tag(&self) -> u32; // 25 bits
    fn get_vpns(&self) -> [u64; 3];
    fn address(&self) -> VAddr;
}

impl SV39Addr for VAddr {
    fn address(&self) -> VAddr {
        *self
    }

    fn tag(&self) -> u32 {
        return ((self >> 39) & 0x1ffffff) as u32;
    }

    fn offset(&self) -> u64 {
        self & 0xfff
    }

    fn level0(&self) -> u64 {
        (self >> 12) & 0x1ff
    }

    fn level1(&self) -> u64 {
        (self >> 21) & 0x1ff
    }

    fn level2(&self) -> u64 {
        (self >> 30) & 0x1ff
    }

    fn get_vpns(&self) -> [u64; 3] {
        [self.level0(), self.level1(), self.level2()]
    }
}

#[repr(u8)]
pub enum PTEPermBit {
    VALID = 0,
    READ = 1,
    WRITE = 2,
    EXECUTE = 3,
    USER = 4,
    GLOBAL = 5,
    ACCESSED = 6,
    DIRTY = 7,
}

type PageTableEntry = u64; // SV39 = u64

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
