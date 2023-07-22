use std::collections::HashSet;

use crate::instructions::Instruction;
use crate::memory::{Memory, MemoryOperations};
use crate::pipeline::{PipelineStages, Stage};

pub type Register = u8;
pub type RegisterValue = u64;

type Registers = [RegisterValue; 32];
type CSRRegisters = [RegisterValue; 4096];

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Xlen {
    Bits32 = 32, // there is really no support for 32-bit only yet
    Bits64 = 64,
    //Bits128 = 128,  // nor any for 128-bit
}

#[derive(Copy, Clone, Debug)]
pub enum PrivMode {
    User,
    Supervisor,
    Machine,
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

#[derive(Debug)]
pub struct Core<'a> {
    pub id: u64,
    pub xlen: Xlen,
    registers: Registers,
    csrs: CSRRegisters,
    pub pmode: PrivMode,
    pub memory: &'a mut dyn MemoryOperations<Memory>,
    pub pc: u64,
    pub prev_pc: u64,
    pub stage: Stage,
    pub cycles: u64,
}

impl<'a> Core<'a> {
    pub fn create(id: u64, memory: &'a mut impl MemoryOperations<Memory>) -> Core<'a> {
        let registers: [RegisterValue; 32] = [0; 32];
        let mut csrs: [RegisterValue; 4096] = [0; 4096];
        csrs[CSRRegister::mhartid as usize] = id;

        Core {
            id,
            xlen: Xlen::Bits64,
            registers,
            csrs,
            pmode: PrivMode::Machine,
            memory,
            pc: 0,
            prev_pc: 0,
            cycles: 0,
            stage: Stage::FETCH,
        }
    }

    pub fn reset(&mut self, pc: u64) {
        self.pc = pc;
        self.stage = Stage::FETCH;
    }

    pub fn pc(&self) -> u64 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u64) {
        self.pc = pc;
    }

    pub fn pmode(&self) -> PrivMode {
        self.pmode
    }

    pub fn read_csr(&self, reg: CSRRegister) -> RegisterValue {
        // SSTATUS, SIE, and SIP are subsets of MSTATUS, MIE, and MIP
        match reg {
            CSRRegister::fflags => self.csrs[CSRRegister::fcsr as usize] & 0x1f,
            CSRRegister::frm => (self.csrs[CSRRegister::fcsr as usize] >> 5) & 0x7,
            CSRRegister::sstatus => self.csrs[CSRRegister::mstatus as usize] & 0x80000003000de162,
            CSRRegister::sie => self.csrs[CSRRegister::mie as usize] & 0x222,
            CSRRegister::sip => self.csrs[CSRRegister::mip as usize] & 0x222,
            // CSRRegister::time => self.mmu.get_clint().read_mtime(),
            _ => self.csrs[reg as usize],
        }
    }

    /// Extends a value depending on XLEN. If in 32-bit mode, will extend the 31st bit
    /// across all bits above it. In 64-bit mode, this is a no-op.
    pub fn bit_extend(&self, value: i64) -> i64 {
        match self.xlen {
            Xlen::Bits32 => value as i32 as i64,
            Xlen::Bits64 => value,
        }
    }

    pub fn write_csr(&mut self, reg: CSRRegister, value: RegisterValue) {
        self.csrs[reg as usize] = value;
    }

    pub fn read_register(&self, reg: Register) -> RegisterValue {
        match reg {
            0 => 0,
            _ => self.registers[reg as usize],
        }
    }

    pub fn write_register(&mut self, reg: Register, value: RegisterValue) {
        if reg != 0 {
            self.registers[reg as usize] = value;
        } else {
            panic!("Should never write to register x0")
        }
    }

    pub fn cycle(&mut self) {
        self.stage = match self.stage {
            Stage::ENTER_TRAP => self.enter_trap(),
            Stage::EXIT_TRAP => self.exit_trap(),
            Stage::FETCH => self.fetch(),
            Stage::DECODE(instruction) => self.decode(&instruction),
            Stage::EXECUTE(decoded) => self.execute(&decoded),
            Stage::MEMORY(memory_access) => self.memory(&memory_access),
            Stage::WRITEBACK(writeback) => self.writeback(writeback),
        };
    }
}
