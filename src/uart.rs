use std::cell::RefCell;

use elfloader::VAddr;

use crate::{cpu::TrapCause, mmio::VirtualDevice, mmu::MemoryRange};

#[derive(Copy, Clone)]
#[repr(u8)]
enum UartIirMask {
    Zero = 0,
    ThrEmpty = 0x2,
    RdAvail = 0x4,
    NoIrq = 0x7,
}

//#[derive(Debug)]
pub struct UART {
    range: MemoryRange,
    state: RefCell<UartState>,
}
//#[derive(Debug)]
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

const IER_RX_ENABLE_BIT: u8 = 0x1;
const IER_TX_ENABLE_BIT: u8 = 0x2;
const LSR_TX_IDLE: u8 = 1 << 5;

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
            eprint!("{}", state.thr as char);
            //            state.terminal.put_byte(state.thr);
            state.thr = 0;
            (_, rx_ip) = UART::update_iir(false, &mut state);
            state.lsr |= LSR_TX_IDLE;
            if (state.ier & IER_TX_ENABLE_BIT) != 0 {
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

    fn update_iir(is_load: bool, state: &mut UartState) -> (u8, bool) {
        let rbr = state.rbr;
        if is_load {
            state.rbr = 0;
            state.lsr &= !0x1; // Data Available
        }

        let rx_ip = (state.ier & IER_RX_ENABLE_BIT) != 0 && state.rbr != 0;
        let thre_ip = (state.ier & IER_TX_ENABLE_BIT) != 0 && state.thr == 0;

        // Which should be prioritized? RX or THRE interrupt?
        if rx_ip {
            state.iir = UartIirMask::RdAvail;
        } else if thre_ip {
            eprint!("thre_ip");
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

    fn write(&mut self, addr: VAddr, value: u8) -> Option<TrapCause> {
        let mut state = self.state.borrow_mut();
        let lcr = state.lcr >> 7;
        let offs = addr - self.range.start;
        match offs {
            0 => match lcr == 0 {
                true => {
                    state.thr = value;
                    //eprintln!("UART out @ {:#x?}: {}", offs, value);
                    UART::update_iir(false, &mut state);
                    state.lsr &= !LSR_TX_IDLE;
                }
                false => {} // @TODO: Implement properly
            },
            0x10000001 => match lcr == 0 {
                true => {
                    // This bahavior isn't written in the data sheet
                    // but some drivers seem to rely on it.
                    if (state.ier & IER_TX_ENABLE_BIT) == 0
                        && (value & IER_TX_ENABLE_BIT) != 0
                        && state.thr == 0
                    {
                        state.thre_ip = true;
                    }
                    state.ier = value;
                    UART::update_iir(false, &mut state);
                    state.lsr &= !LSR_TX_IDLE;
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
        None
    }

    //    fn

    fn read(&mut self, addr: VAddr) -> Result<u8, TrapCause> {
        let mut state = self.state.borrow_mut();
        let lcr = state.lcr;
        let offs = addr - self.range.start;
        Ok(match offs {
            0 => match (lcr >> 7) == 0 {
                true => UART::update_iir(true, &mut state).0,
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
        })
    }
}
