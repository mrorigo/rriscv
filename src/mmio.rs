use std::cell::RefCell;

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

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum UartIirMask {
    Zero = 0,
    ThrEmpty = 0x2,
    RdAvail = 0x4,
    NoIrq = 0x7,
}

#[derive(Debug)]
pub struct UART {
    range: MemoryRange,
    state: RefCell<UartState>,
}
#[derive(Debug)]
pub struct UartState {
    clock: u64,
    rbr: u8,          // receiver buffer register
    thr: u8,          // transmitter holding register
    ier: u8,          // interrupt enable register
    iir: UartIirMask, // interrupt identification register
    lcr: u8,          // line control register
    mcr: u8,          // modem control register
    lsr: u8,          // line status register
    scr: u8,          // scratch,
    thre_ip: bool,
    interrupting: bool,
}

const IER_RXINT_BIT: u8 = 0x1;
const IER_THREINT_BIT: u8 = 0x2;
const LSR_THR_EMPTY: u8 = 0x20;

impl UART {
    pub fn create(range: MemoryRange) -> UART {
        UART {
            range,
            state: RefCell::new(UartState {
                clock: 0,
                rbr: 0,
                thr: 0,
                ier: 0,
                iir: UartIirMask::Zero,
                lcr: 0,
                mcr: 0,
                lsr: 0,
                scr: 0,
                thre_ip: false,
                interrupting: false,
            }),
        }
    }

    pub fn tick(&mut self) {
        let mut state = self.state.borrow_mut();
        state.clock = state.clock.wrapping_add(1);
        let mut rx_ip = false;
        if (state.clock % 0x10) == 0 && state.thr != 0 {
            println!("terminal.put_byte: {}", state.thr);
            //            state.terminal.put_byte(state.thr);
            state.thr = 0;
            state.lsr |= LSR_THR_EMPTY;
            (_, rx_ip) = UART::update_iir(false, false, &mut state);
            if (state.ier & IER_THREINT_BIT) != 0 {
                state.thre_ip = true;
            }
        }

        if state.thre_ip || rx_ip {
            state.interrupting = true;
            state.thre_ip = false;
        } else {
            state.interrupting = false;
        }
    }

    fn update_iir(is_load: bool, is_store: bool, state: &mut UartState) -> (u8, bool) {
        let rbr = state.rbr;
        if is_load {
            state.rbr = 0;
            state.lsr &= !0x1; // Data Available
        } else if is_store {
            state.lsr &= !LSR_THR_EMPTY;
        }

        let rx_ip = (state.ier & IER_RXINT_BIT) != 0 && state.rbr != 0;
        let thre_ip = (state.ier & IER_THREINT_BIT) != 0 && state.thr == 0;

        // Which should be prioritized RX interrupt or THRE interrupt?
        if rx_ip {
            state.iir = UartIirMask::RdAvail;
        } else if thre_ip {
            state.iir = UartIirMask::ThrEmpty;
        } else {
            state.iir = UartIirMask::NoIrq;
        }

        (rbr, rx_ip)
    }
}

impl VirtualDevice for UART {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }

    fn write(&mut self, addr: VAddr, value: u8) -> bool {
        let mut state = self.state.borrow_mut();
        let lcr = state.lcr >> 7;
        let offs = addr - self.range.start;
        println!("UART out @ {:#x?}: {}", offs, value);
        match offs {
            0 => match lcr == 0 {
                true => {
                    state.thr = value;
                    UART::update_iir(false, true, &mut state);
                }
                false => {} // @TODO: Implement properly
            },
            0x10000001 => match lcr == 0 {
                true => {
                    // This bahavior isn't written in the data sheet
                    // but some drivers seem to rely on it.
                    if (state.ier & IER_THREINT_BIT) == 0
                        && (value & IER_THREINT_BIT) != 0
                        && state.thr == 0
                    {
                        state.thre_ip = true;
                    }
                    state.ier = value;
                    UART::update_iir(false, true, &mut state);
                }
                false => {} // @TODO: Implement properly
            },
            0x10000003 => {
                state.lcr = value;
            }
            0x10000004 => {
                state.mcr = value;
            }
            0x10000007 => {
                state.scr = value;
            }
            _ => {}
        }
        //panic!("UART::write");
        true
    }

    //    fn

    fn read(&self, addr: VAddr) -> u8 {
        let mut state = self.state.borrow_mut();
        let lcr = state.lcr;
        let offs = addr - self.range.start;
        match offs {
            0 => match (lcr >> 7) == 0 {
                true => UART::update_iir(true, false, &mut state).0,
                false => 0, // @TODO: DLL divisor latch LSB
            },
            1 => match (lcr >> 7) == 0 {
                true => state.ier,
                false => 0, // @TODO: DLM divisor latch MSB
            },
            2 => state.iir as u8,
            3 => state.lcr,
            4 => state.mcr,
            5 => state.lsr,
            7 => state.scr,
            _ => 0,
        }
    }
}

pub struct MMIODevice {}

impl MMIODevice {
    // pub fn create(range: MemoryRange) -> Box<dyn VirtualDevice> {
    //     match range.name {
    //         "memory" => Box::new(PhysicalMemory::create(range)),
    //         "uart" => Box::new(UART::create(range)),
    //         _ => panic!(),
    //     }
    // }
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
    fn write(&mut self, addr: VAddr, value: u8) -> bool;
    fn read(&self, addr: VAddr) -> u8;
    fn tick(&mut self) {}
}

// impl MemoryOperations<UART, u8> for UART {
//     fn read_single(&self, addr: VAddr) -> Option<u8> {
//         mmio_trace!(println!("UART: read_single {:#x?}", addr));
//         None
//     }
//     fn write_single(&mut self, addr: VAddr, value: u8) -> bool {
//         mmio_trace!(println!("UART: write_single {:#x?} @ {:#x?}", value, addr));
//         true
//     }
// }

impl MemoryOperations<PhysicalMemory, u8> for PhysicalMemory {
    fn read_single(&self, addr: VAddr) -> Option<u8> {
        self.ram.read_single(addr)
    }

    fn write_single(&mut self, addr: VAddr, value: u8) -> bool {
        self.ram.write_single(addr, value)
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
        self.ram.write_single(addr, value)
    }

    fn read(&self, addr: VAddr) -> u8 {
        self.ram.read_single(addr).unwrap()
    }
}

impl MemoryOperations<CLINT, u8> for CLINT {
    fn read_single(&self, addr: VAddr) -> Option<u8> {
        self.ram.read_single(addr)
    }

    fn write_single(&mut self, addr: VAddr, value: u8) -> bool {
        self.ram.write_single(addr, value)
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
        self.ram.write_single(addr, value)
    }

    fn read(&self, addr: VAddr) -> u8 {
        self.ram.read_single(addr).unwrap()
    }
}

impl VirtualDevice for VIRTIO {
    fn includes(&self, addr: VAddr) -> bool {
        self.range.includes(addr)
    }

    fn name(&self) -> &str {
        self.range.name
    }
    fn write(&mut self, _addr: VAddr, _value: u8) -> bool {
        todo!()
    }

    fn read(&self, _addr: VAddr) -> u8 {
        todo!()
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
