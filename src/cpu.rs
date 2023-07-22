use crate::decoder::{
    Btype, CBtype, CIWtype, CItype, CJtype, CLtype, CRtype, CSStype, CStype, Itype, Jtype, Rtype,
    Stype, Utype,
};

use crate::memory::MemoryOperations;
use crate::opcodes;
use crate::pipeline::{PipelineStages, Stage, WritebackStage};

#[derive(Copy, Clone, Debug)]
pub enum PrivMode {
    User,
    Supervisor,
    Machine,
}

pub type Register = u8;
pub type RegisterValue = u64;

type Registers = [RegisterValue; 32];
type CSRRegisters = [RegisterValue; 4096];

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, FromPrimitive)]
#[repr(u16)]
pub enum CSRRegister {
    mvendorid = 0xf11,
    marchid = 0xf11 + 1,
    mimpid = 0xf11 + 2,
    mhartid = 0xf11 + 3,
    mconfigptr = 0xf11 + 4,
    mstatus = 0x300,
    misa = 0x300 + 1,
    medeleg = 0x300 + 2,
    mideleg = 0x300 + 3,
    mie = 0x300 + 4,
    mtvec = 0x300 + 5,
    mcounteren = 0x300 + 6,
    mstatush = 0x300 + 7,

    mcycle = 0xb00,
    minstret = 0xb02,

    minstreth = 0xb82, // RV32 only
}

#[derive(Debug)]
pub struct Core<'a> {
    pub id: u64,
    pub registers: Registers,
    pub csrs: CSRRegisters,
    pub pmode: PrivMode,
    pub memory: &'a mut dyn MemoryOperations,
    pub pc: u64,
    pub prev_pc: u64,
    pub stage: Stage,
    pub cycles: u64,
}

impl<'a> Core<'a> {
    pub fn create(id: u64, memory: &'a mut impl MemoryOperations) -> Core<'a> {
        let registers: [RegisterValue; 32] = [0; 32];
        let mut csrs: [RegisterValue; 4096] = [0; 4096];
        csrs[CSRRegister::mhartid as usize] = id;

        Core {
            id,
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

pub trait Execution {
    fn execute(&self, i: &Core) -> Stage;
}

impl Execution for Itype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for Jtype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for Utype {
    fn execute(&self, core: &Core) -> Stage {
        match self.opcode {
            opcodes::MajorOpcode::AUIPC => {
                const M: u32 = 1 << (20 - 1);
                let se_imm20 = (self.imm20 ^ M) - M;
                Stage::WRITEBACK(Some(WritebackStage {
                    register: self.rd,
                    value: (se_imm20 << 12) as u64 + core.prev_pc,
                }))
            }
            _ => panic!(),
        }
    }
}

impl Execution for CRtype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for Btype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for Stype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for Rtype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for CItype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for CSStype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for CIWtype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for CLtype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for CStype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for CBtype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}

impl Execution for CJtype {
    fn execute(&self, i: &Core) -> Stage {
        match self.opcode {
            _ => panic!(),
        }
    }
}
