use std::collections::{HashMap, VecDeque};

use elfloader::VAddr;

use crate::mmu::MMU;
use crate::pipeline::{PipelineStages, Stage};

pub type Register = u8;
pub type RegisterValue = u64;

type Registers = [RegisterValue; 32];
type CSRRegisters = [RegisterValue; 4096];

macro_rules! cpu_trace {
    ($instr:expr) => {
        print!("C:");
        $instr;
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum TrapCause {
    SupervisorSoftlIrq = 0x101,
    MachineSoftlIrq = 0x103,
    SupervisorTimerlIrq = 0x105,
    MachineTimerlIrq = 0x107,
    SupervisorExternallIrq = 0x109,
    MachineExternalIrq = 0x10B,

    InstructionMisaligned = 0x00,
    InstructionAccessFault = 0x01,
    IllegalInstruction = 0x02,
    Breakpoint = 0x03,
    LoadAddressMisaligned = 0x04,
    LoadAccessFault = 0x05,

    EnvCallFromUmode = 0x08,
    EnvCallFromSmode = 0x09,
    EnvCallFromMmode = 0x0B,

    InstructionPageFault = 0x0C,
    LoadPageFault = 0x0D,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Xlen {
    Bits32 = 32, // there is really no support for 32-bit only yet
    Bits64 = 64,
    //Bits128 = 128,  // nor any for 128-bit
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u8)]
pub enum PrivMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, FromPrimitive)]
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

pub struct Core {
    pub id: u64,
    pub xlen: Xlen,
    registers: Registers,
    csrs: CSRRegisters,
    pmode: PrivMode,
    pc: u64,
    pub prev_pc: u64,
    pub stage: Stage,
    pub cycles: u64,
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

    pub fn pc(&self) -> u64 {
        self.pc
    }

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

    pub fn set_pc(&mut self, pc: u64) {
        let symbol = self.find_closest_symbol(pc);
        let last_st = self.symboltrace.back();
        match symbol {
            Some(sym) => {
                cpu_trace!(println!("set_pc = {:#x?}  symbol = {:?}", pc, sym.1));
                match last_st {
                    Some((_last_addr, last_symbol)) => {
                        if sym.0 == pc && last_symbol.ne(&sym.1) {
                            //println!("Push symboltrace: {:?}@{:#x?}", sym.1, sym.0);
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
        // assert!(
        //     name != "end",
        //     "Reached 'end' symbol. This usually means your program is over. Be happy!"
        // );
        self.pc = pc;
    }

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
            CSRRegister::time => todo!(),

            _ => {
                //cpu_trace!(println!("write_csr {:#x?} = {:#x?}", reg, value));
                self.csrs[reg as usize] = value;
            }
        }
    }

    pub fn read_register(&self, reg: Register) -> RegisterValue {
        let v = match reg {
            0 => 0,
            _ => self.registers[reg as usize],
        };
        //cpu_trace!(println!("read_register x{:#?} = {:#x?}", reg, v));
        v
    }

    pub fn write_register(&mut self, reg: Register, value: RegisterValue) {
        if reg != 0 {
            //cpu_trace!(println!("write_register x{:#?} = {:#x?}", reg, value));
            self.registers[reg as usize] = value;
        } else {
            panic!("Should never write to register x0")
        }
    }

    pub fn cycle(&mut self, mmu: &mut MMU) {
        cpu_trace!(println!("stage: {:?}", self.stage));
        self.stage = match self.stage {
            Stage::ENTER_TRAP(cause) => self.enter_trap(cause),
            Stage::EXIT_TRAP => self.exit_trap(),
            Stage::FETCH => self.fetch(mmu),
            Stage::DECODE(instruction) => self.decode(&instruction),
            Stage::EXECUTE(decoded) => self.execute(&decoded),
            Stage::MEMORY(memory_access) => self.memory(mmu, &memory_access),
            Stage::WRITEBACK(writeback) => self.writeback(writeback),
        };
    }
}
