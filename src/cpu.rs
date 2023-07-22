use std::collections::{HashMap, VecDeque};
use std::usize;

use elfloader::VAddr;

use crate::debugger::{Debugger, DebuggerResult};
use crate::mmu::MMU;
use crate::pipeline::{PipelineStages, Stage};

pub type Register = u8;
pub type RegisterValue = u64;

type Registers = [RegisterValue; 32];
type CSRRegisters = [RegisterValue; 4096];

macro_rules! cpu_trace {
    ($instr:expr) => {
        // print!("C:");
        // $instr;
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum TrapCause {
    UserSoftwareIrq = 0x100,
    SupervisorSoftIrq = 0x101,
    MachineSoftIrq = 0x103,
    UserTimerIrq = 0x104,
    SupervisorTimerIrq = 0x105,
    MachineTimerIrq = 0x107,
    UserExternalIrq = 0x108,
    SupervisorExternalIrq = 0x109,
    MachineExternalIrq = 0x10B,

    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault(VAddr) = 5,
    StoreAddressMisaligned = 6,
    StoreAccessFault(VAddr) = 7,
    EnvCallFromUMode = 8,
    EnvCallFromSMode = 9,
    EnvCallFromMMode = 11,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
}

impl From<TrapCause> for u16 {
    fn from(value: TrapCause) -> Self {
        match value {
            TrapCause::InstructionAddressMisaligned => 0,
            TrapCause::InstructionAccessFault => 1,
            TrapCause::IllegalInstruction => 2,
            TrapCause::Breakpoint => 3,
            TrapCause::LoadAddressMisaligned => 4,
            TrapCause::LoadAccessFault(_) => 5,
            TrapCause::StoreAddressMisaligned => 6,
            TrapCause::StoreAccessFault(_) => 7,
            TrapCause::EnvCallFromUMode => 8,
            TrapCause::EnvCallFromSMode => 9,
            TrapCause::EnvCallFromMMode => 11,
            TrapCause::InstructionPageFault => 12,
            TrapCause::LoadPageFault => 13,
            TrapCause::StorePageFault => 15,
            TrapCause::UserSoftwareIrq => 0x100,
            TrapCause::SupervisorSoftIrq => 0x101,
            TrapCause::MachineSoftIrq => 0x103,
            TrapCause::UserTimerIrq => 0x104,
            TrapCause::SupervisorTimerIrq => 0x105,
            TrapCause::MachineTimerIrq => 0x107,
            TrapCause::UserExternalIrq => 0x108,
            TrapCause::SupervisorExternalIrq => 0x109,
            TrapCause::MachineExternalIrq => 0x10B,
        }
    }
}

impl TrapCause {}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Xlen {
    Bits32 = 32, // there is really no support for 32-bit only yet
    Bits64 = 64,
    //Bits128 = 128,  // nor any for 128-bit
}

#[derive(Clone, Copy, PartialEq, Debug, FromPrimitive, PartialOrd)]
#[repr(u8)]
pub enum PrivMode {
    User = 0,
    Supervisor = 1,
    Reserved = 2,
    Machine = 3,
}

impl From<PrivMode> for u64 {
    fn from(value: PrivMode) -> Self {
        match value {
            PrivMode::User => 0,
            PrivMode::Supervisor => 1,
            PrivMode::Reserved => 2,
            PrivMode::Machine => 3,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
#[repr(u16)]
pub enum CSRRegister {
    ustatus = 0,
    fflags = 1,
    frm = 2,
    fcsr = 3,
    uie = 4,
    utvec = 5,

    uepc = 0x41,
    ucause = 0x42,
    utval = 0x43,

    sstatus = 0x100,
    sedeleg = 0x102,
    sideleg = 0x103,
    sie = 0x104,
    stvec = 0x105,

    sscratch = 0x140,
    sepc = 0x141,
    scause = 0x142,
    stval = 0x143,
    sip = 0x144,

    satp = 0x180,

    mstatus = 0x300,
    misa = 0x301,
    medeleg = 0x302,
    mideleg = 0x303,
    mie = 0x304,
    mtvec = 0x305,
    mcounteren = 0x306,
    mstatush = 0x307,

    mscratch = 0x340,
    mepc = 0x341,
    mcause = 0x342,
    mtval = 0x343,
    mip = 0x344,

    pmpcfg0 = 0x3a0,
    pmpaddr0 = 0x3b0,

    debug0 = 0x7a0,
    debug1 = 0x7a1,
    debug2 = 0x7a2,
    debug3 = 0x7a3,
    debug4 = 0x7a4,
    debug5 = 0x7a5,
    debug6 = 0x7a6,
    debug7 = 0x7a7,
    debug8 = 0x7a8,

    mcycle = 0xb00,
    minstret = 0xb02,

    minstreth = 0xb82, // RV32 only

    cycle = 0xc00,
    time = 0xc01,
    instret = 0xc02,

    cycleh = 0xc80,
    timeh = 0xc81,
    instreth = 0xc82,

    mvendorid = 0xf11,
    marchid = 0xf12,
    mimpid = 0xf13,
    mhartid = 0xf14,
    mconfigptr = 0xf15,
}

pub const CYCLES_PER_INSTRUCTION: usize = 5;

pub struct Core {
    pub id: u64,
    pub xlen: Xlen,
    registers: Registers,
    csrs: CSRRegisters,
    pmode: PrivMode,
    pc: u64,
    wfi: bool,
    pub prev_pc: u64,
    pub stage: Stage,
    pub cycles: u64,
    // Debug usage:
    step_cycles: usize,
    breakpoint_address: Option<VAddr>,
    pub symbols: HashMap<VAddr, String>,
    pub symboltrace: VecDeque<(VAddr, String)>,
}

impl Core {
    pub fn create(id: u64) -> Core {
        let registers: [RegisterValue; 32] = [0; 32];
        let mut csrs: [RegisterValue; 4096] = [0; 4096];
        csrs[CSRRegister::mhartid as usize] = id;

        Core {
            id,
            xlen: Xlen::Bits64,
            registers,
            csrs,
            pmode: PrivMode::Machine,
            pc: 0,
            prev_pc: 0,
            cycles: 0,
            step_cycles: 0,
            breakpoint_address: None,
            wfi: false,
            stage: Stage::FETCH,
            symbols: HashMap::new(),
            symboltrace: VecDeque::<(VAddr, String)>::new(),
        }
    }

    pub fn add_symbol(&mut self, addr: VAddr, name: String) {
        //println!("cpu: Adding symbol {:?} = {:#x?}", name, addr);
        self.symbols.insert(addr, name);
    }

    pub fn reset(&mut self, pc: u64) {
        self.pc = pc;
        self.stage = Stage::FETCH;
    }

    #[inline]
    pub fn pc(&self) -> u64 {
        self.pc
    }

    #[inline]
    pub fn add_pc(&mut self, offs: u64) {
        self.pc += offs
    }

    //#[allow(dead_code)]
    pub fn find_closest_symbol(&self, addr: VAddr) -> Option<(u64, String)> {
        match self.symbols.get(&addr) {
            Some(symbol) => Some((addr, symbol.clone())),
            None => {
                None
                // let unknown_str = "<unknown>".to_string();
                // let unknown = Some(unknown_str);
                // let mut closest = None;
                // let mut closest_dist = 1e6 as u64;
                // for sym in self.symbols.iter() {
                //     let dist = i64::abs(addr as i64 - *sym.0 as i64) as u64;
                //     if dist < closest_dist && dist > 0 && *sym.0 < addr {
                //         closest = Some(sym.clone()); //sym.1.clone());
                //         closest_dist = dist;
                //     }
                // }
                // if closest.is_some() {
                //     Some((*closest.unwrap().0, closest.unwrap().1.clone()))
                // } else {
                //     None
                // }
            }
        }
    }

    pub fn debug_breakpoint(&mut self, cause: TrapCause, mmu: &mut MMU) {
        let debugger = Debugger::create();
        match debugger.enter(self, mmu, cause) {
            DebuggerResult::Continue => {}
            DebuggerResult::ContinueUntil(bp_addr) => self.breakpoint_address = Some(bp_addr),
            DebuggerResult::Step(_nsteps) => self.step_cycles = _nsteps,
            DebuggerResult::Quit(reason) => panic!("Quitting: {:?}", reason),
        }
    }

    pub fn set_pc(&mut self, pc: u64) {
        if self.symbols.len() > 0 {
            let symbol = self.find_closest_symbol(pc);
            let last_st = self.symboltrace.back();
            match symbol {
                Some(sym) => {
                    // cpu_trace!(println!("set_pc = {:#x?}  symbol = {:?}", pc, sym.1));
                    match last_st {
                        Some((_last_addr, last_symbol)) => {
                            if sym.0 == pc && last_symbol.ne(&sym.1) {
                                //println!("Push symboltrace: {:?}@{:#x?}", sym.1, sym.0);
                                cpu_trace!(println!("set_pc = {:#x?}  symbol = {:?}", pc, sym.1));
                                self.symboltrace.push_back((pc, sym.1));
                                if self.symboltrace.len() > 20 {
                                    self.symboltrace.pop_front();
                                }
                            }
                        }
                        None => {
                            self.symboltrace.push_back((pc, sym.1));
                        }
                    }
                }
                None => {}
            }
        }
        self.pc = pc;
    }

    #[inline]
    pub fn pmode(&self) -> PrivMode {
        self.pmode
    }

    pub fn set_pmode(&mut self, pmode: PrivMode) -> PrivMode {
        let ret = self.pmode;
        cpu_trace!(println!("set_pmode = {:#x?} (was {:#x?})", pmode, ret));
        self.pmode = pmode;
        ret
    }

    /// Extends a value depending on XLEN. If in 32-bit mode, will extend the 31st bit
    /// across all bits above it. In 64-bit mode, this is a no-op.
    #[inline]
    pub fn bit_extend(&self, value: i64) -> i64 {
        let res = match self.xlen {
            Xlen::Bits32 => ((value as i32) as u32 & 0xffffffff) as i64,
            Xlen::Bits64 => value,
        };
        //println!("bit_extend ({:?}) {:#x?} => {:#x?}", self.xlen, value, res);
        res
    }

    pub fn read_csr(&self, reg: CSRRegister) -> RegisterValue {
        // SSTATUS, SIE, and SIP are subsets of MSTATUS, MIE, and MIP
        let value = match reg {
            CSRRegister::fflags => self.csrs[CSRRegister::fcsr as usize] & 0x1f,
            CSRRegister::frm => (self.csrs[CSRRegister::fcsr as usize] >> 5) & 0x7,
            //                               UXL
            // 10000000000000000000000000000011 00000000 00001101 11100000 00000000
            CSRRegister::sstatus => self.csrs[CSRRegister::mstatus as usize] & 0x80000003000de162,
            CSRRegister::sie => self.csrs[CSRRegister::mie as usize] & 0x222,
            CSRRegister::sip => self.csrs[CSRRegister::mip as usize] & 0x222,
            CSRRegister::time => todo!(),
            _ => self.csrs[reg as usize],
        };
        //cpu_trace!(println!("read_csr {:#x?} = {:#x?}", reg, value));

        value
    }

    pub fn write_csr(&mut self, reg: CSRRegister, value: RegisterValue) {
        let old = self.csrs[reg as usize];
        match reg {
            CSRRegister::fflags => {
                self.csrs[CSRRegister::fcsr as usize] &= !0x1f;
                self.csrs[CSRRegister::fcsr as usize] |= value & 0x1f;
            }
            CSRRegister::frm => {
                self.csrs[CSRRegister::fcsr as usize] &= !0xe0;
                self.csrs[CSRRegister::fcsr as usize] |= (value << 5) & 0xe0;
            }
            CSRRegister::sstatus => {
                self.csrs[CSRRegister::mstatus as usize] &= !0x80000003000de162;
                self.csrs[CSRRegister::mstatus as usize] |= value & 0x80000003000de162;
            }
            CSRRegister::sie => {
                self.csrs[CSRRegister::mie as usize] &= !0x222;
                self.csrs[CSRRegister::mie as usize] |= value & 0x222;
            }
            CSRRegister::sip => {
                self.csrs[CSRRegister::mip as usize] &= !0x222;
                self.csrs[CSRRegister::mip as usize] |= value & 0x222;
            }
            CSRRegister::mideleg => {
                self.csrs[reg as usize] = value & 0x666; // from qemu
            }
            CSRRegister::mstatus => {
                self.csrs[CSRRegister::mstatus as usize] &= !0x80000003000de162;
                self.csrs[CSRRegister::mstatus as usize] |= value & 0x80000003000de162;
            }
            CSRRegister::time => todo!(),

            _ => {
                self.csrs[reg as usize] = value;
            }
        }
        // if reg == CSRRegister::stvec {
        //     if self.csrs[reg as usize] != old {
        //         println!(
        //             "-> {:?} csr set: {:#x?}  ({:#x?})",
        //             reg, self.csrs[reg as usize], value
        //         );
        //     }
        // }

        //cpu_trace!(println!("write_csr {:#x?} = {:#x?}", reg, value));
    }

    #[inline]
    pub fn read_register(&self, reg: Register) -> RegisterValue {
        let v = match reg {
            0 => 0,
            _ => self.registers[reg as usize],
        };
        //cpu_trace!(println!("read_register x{:#?} = {:#x?}", reg, v));
        v
    }

    #[inline]
    pub fn write_register(&mut self, reg: Register, value: RegisterValue) {
        if reg != 0 {
            self.registers[reg as usize] = value;
        } else {
            panic!("Should never write to register x0")
        }
    }

    pub fn cycle(&mut self, mmu: &mut MMU) {
        if self.step_cycles > 0 {
            self.step_cycles = self.step_cycles - 1;
            if self.step_cycles == 0 {
                self.debug_breakpoint(TrapCause::Breakpoint, mmu);
            }
        }
        match self.breakpoint_address {
            None => {}
            Some(addr) => {
                if self.pc == addr {
                    self.debug_breakpoint(TrapCause::Breakpoint, mmu);
                }
            }
        }
        //cpu_trace!(println!("stage: {:?}", self.stage));
        self.stage = match self.stage {
            Stage::TRAP(cause) => self.trap(cause),
            Stage::FETCH => self.fetch(mmu),
            Stage::DECODE(instruction) => self.decode(&instruction),
            Stage::EXECUTE(decoded) => self.execute(&decoded),
            Stage::MEMORY(memory_access) => self.memory(mmu, &memory_access),
            Stage::WRITEBACK(writeback) => self.writeback(writeback),
            Stage::IRQ => match self.handle_interrupts() {
                Some(cause) => Stage::TRAP(cause),
                _ => Stage::FETCH,
            },
        };
        mmu.update_privilege_mode(self.pmode);
        mmu.update_satp(self.read_csr(CSRRegister::satp), self.xlen);
        mmu.update_mstatus(self.read_csr(CSRRegister::mstatus));
    }

    //
    fn handle_interrupts(&mut self) -> Option<TrapCause> {
        let mie = self.read_csr(CSRRegister::mie);
        let mip = self.read_csr(CSRRegister::mip);
        if mip == 0 {
            return None;
        }
        let minterrupt = mip & mie;
        if MipMask::has_seip_set(minterrupt) {
            return Some(TrapCause::SupervisorExternalIrq);
        } else if MipMask::has_meip_set(minterrupt) {
            return Some(TrapCause::MachineExternalIrq);
        } else if MipMask::has_msip_set(minterrupt) {
            return Some(TrapCause::MachineSoftIrq);
        } else if MipMask::has_mtip_set(minterrupt) {
            return Some(TrapCause::MachineTimerIrq);
        } else if MipMask::has_ssip_set(minterrupt) {
            return Some(TrapCause::SupervisorSoftIrq);
        } else if MipMask::has_stip_set(minterrupt) {
            return Some(TrapCause::SupervisorTimerIrq);
        }

        // self.write_csr(
        //     CSRRegister::mip,
        //     self.read_csr(CSRRegister::mip) & !MIP_SEIP,
        // );
        // self.wfi = false;
        // return;
        None
    }

    // Update the instret CSR based on what PrivMode we are in
    pub fn update_instret(&mut self) {
        let (instretcsr, instrethcsr) = match self.pmode() {
            PrivMode::Machine => (CSRRegister::minstret, CSRRegister::minstreth),
            PrivMode::Supervisor => (CSRRegister::instret, CSRRegister::instreth),
            PrivMode::User => (CSRRegister::instret, CSRRegister::instreth),
            _ => panic!("Unknown privilege mode. We should not be here."),
        };

        let instret = self.read_csr(instretcsr);
        if instret.wrapping_add(1) < instret {
            self.write_csr(instrethcsr, self.read_csr(instrethcsr).wrapping_add(1));
        }
        self.write_csr(instretcsr, instret.wrapping_add(1));
    }

    /// Map TrapCause to a `mcause` CSR register value
    pub fn get_mcause(&self, xlen: Xlen, value: TrapCause) -> u64 {
        let interrupt_bit = match xlen {
            Xlen::Bits32 => 0x80000000 as u64,
            Xlen::Bits64 => 0x8000000000000000 as u64,
        };
        match value {
            TrapCause::InstructionAddressMisaligned
            | TrapCause::InstructionAccessFault
            | TrapCause::IllegalInstruction
            | TrapCause::Breakpoint
            | TrapCause::LoadAddressMisaligned
            | TrapCause::LoadAccessFault(_)
            | TrapCause::StoreAddressMisaligned
            | TrapCause::StoreAccessFault(_)
            | TrapCause::EnvCallFromUMode
            | TrapCause::EnvCallFromSMode
            | TrapCause::EnvCallFromMMode
            | TrapCause::InstructionPageFault
            | TrapCause::LoadPageFault
            | TrapCause::StorePageFault => u16::from(value) as u64,

            TrapCause::UserSoftwareIrq
            | TrapCause::SupervisorSoftIrq
            | TrapCause::MachineSoftIrq
            | TrapCause::UserTimerIrq
            | TrapCause::SupervisorTimerIrq
            | TrapCause::MachineTimerIrq
            | TrapCause::UserExternalIrq
            | TrapCause::SupervisorExternalIrq
            | TrapCause::MachineExternalIrq => (u16::from(value) - 0x100) as u64 + interrupt_bit,
        }
    }
}

// #[derive(Copy, Clone)]
// #[repr(u16)]
// Masks for the `mip` CSR register
#[non_exhaustive]
pub struct MipMask {}
impl MipMask {
    pub const SSIP: u16 = 0x002;
    pub const MSIP: u16 = 0x008;
    pub const STIP: u16 = 0x020;
    pub const MTIP: u16 = 0x080;
    pub const SEIP: u16 = 0x200;
    pub const MEIP: u16 = 0x800;
}

impl From<MipMask> for u64 {
    fn from(value: MipMask) -> Self {
        value.into()
    }
}

macro_rules! mip_bit_check_impl {
    ($name:ident,$mask:expr) => {
        pub fn $name(value: u64) -> bool {
            (value as u16 & $mask as u16) != 0
        }
    };
}
impl MipMask {
    mip_bit_check_impl!(has_msip_set, MipMask::MSIP);
    mip_bit_check_impl!(has_stip_set, MipMask::STIP);
    mip_bit_check_impl!(has_ssip_set, MipMask::SSIP);
    mip_bit_check_impl!(has_mtip_set, MipMask::MTIP);
    mip_bit_check_impl!(has_seip_set, MipMask::SEIP);
    mip_bit_check_impl!(has_meip_set, MipMask::MEIP);
}
