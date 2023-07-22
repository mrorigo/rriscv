use crate::memory::MemoryOperations;
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

    sip = 0x144,

    mstatus = 0x300,
    misa = 0x301,
    medeleg = 0x302,
    mideleg = 0x303,
    mie = 0x304,
    mtvec = 0x305,
    mcounteren = 0x306,
    mstatush = 0x307,

    mip = 0x344,

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

// pub trait ExecutionStage {
//     fn execute(&self, _core: &mut Core) -> Stage;
// }

// impl ExecutionStage for Itype {
//     fn execute(&self, core: &mut Core) -> Stage {
//         let value = match self.opcode {
//             MajorOpcode::OP_IMM => {
//                 let rs1v = core.read_register(self.rs1);
//                 match num::FromPrimitive::from_u8(self.funct3) {
//                     Some(OpImmFunct3::ADDI) => {
//                         Some(rs1v.wrapping_add((self.imm12 as u64).sign_extend(64 - 12)))
//                     }
//                     _ => panic!(),
//                 }
//             }
//             MajorOpcode::SYSTEM => {
//                 let csr_register = num::FromPrimitive::from_u16(self.imm12).unwrap();
//                 let csrv = core.read_csr(csr_register);
//                 match num::FromPrimitive::from_u8(self.funct3) {
//                     Some(CSR_Funct3::CSRRS) => {
//                         // For both CSRRS and CSRRC, if rs1=x0, then the instruction will not write to the CSR at all
//                         if self.rs1 != 0 {
//                             core.write_csr(csr_register, csrv | core.read_register(self.rs1));
//                         }
//                         Some(csrv)
//                     }
//                     _ => panic!(),
//                 }
//             }

//             _ => panic!(),
//         };

//         if value.is_some() {
//             Stage::WRITEBACK(Some(WritebackStage {
//                 register: self.rd,
//                 value: value.unwrap(),
//             }))
//         } else {
//             Stage::WRITEBACK(None)
//         }
//     }
// }

// impl ExecutionStage for Jtype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             MajorOpcode::JAL => {
//                 const M: u32 = 1 << (20 - 1);
//                 let se_imm20 = ((self.imm20 << 1) ^ M) - M;
//                 println!("core.pc: {:#x?}  se_imm20: {:#x?}", _core.pc, se_imm20);
//                 _core.set_pc(_core.prev_pc + se_imm20 as u64);
//                 Stage::WRITEBACK(None)
//             }
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for Utype {
//     fn execute(&self, core: &mut Core) -> Stage {
//         match self.opcode {
//             MajorOpcode::AUIPC => {
//                 const M: u32 = 1 << (20 - 1);
//                 let se_imm20 = (self.imm20 ^ M) - M;
//                 Stage::WRITEBACK(Some(WritebackStage {
//                     register: self.rd,
//                     value: (se_imm20 << 12) as u64 + core.prev_pc,
//                 }))
//             }
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for Btype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for Stype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for Rtype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         let value = match self.opcode {
//             MajorOpcode::OP => match self.funct7 {
//                 // RV32M
//                 1 => {
//                     let r1v = _core.read_register(self.rs1);
//                     let r2v = _core.read_register(self.rs2);
//                     match num::FromPrimitive::from_u8(self.funct3).unwrap() {
//                         RV32M_Funct3::MUL => {
//                             Some(_core.bit_extend(r1v.wrapping_mul(r2v) as i64) as u64)
//                         }
//                         RV32M_Funct3::MULH => match _core.xlen {
//                             Xlen::Bits32 => {
//                                 Some(_core.bit_extend((r1v as i64 * r2v as i64) >> 32) as u64)
//                             }
//                             Xlen::Bits64 => Some(((r1v as i128) * (r2v as i128) >> 64) as u64),
//                         },
//                         _ => panic!(),
//                     }
//                 }
//                 _ => todo!("Support non-RV32M OP opcode"),
//             },
//             _ => panic!(),
//         };
//         if value.is_some() {
//             Stage::WRITEBACK(Some(WritebackStage {
//                 register: self.rd,
//                 value: value.unwrap() as u64,
//             }))
//         } else {
//             Stage::WRITEBACK(None)
//         }
//     }
// }

// impl ExecutionStage for CItype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             opcodes::CompressedOpcode::C1 => {
//                 let rs1v = _core.read_register(self.rd);
//                 // @TODO: enum
//                 let value = match num::FromPrimitive::from_u8(self.funct3).unwrap() {
//                     C1_Funct3::C_LUI => Some(((self.imm as u64) << 12).sign_extend(64 - 17)),
//                     C1_Funct3::C_LI => Some((self.imm as u64).sign_extend(64 - 6)),
//                     C1_Funct3::C_ADDI => match self.rd {
//                         // NOP
//                         0 => None,
//                         _ => Some(rs1v.wrapping_add((self.imm as u64).sign_extend(64 - 6))),
//                     },
//                     _ => panic!(),
//                 };
//                 if value.is_some() {
//                     Stage::WRITEBACK(Some(WritebackStage {
//                         register: self.rd,
//                         value: value.unwrap(),
//                     }))
//                 } else {
//                     Stage::WRITEBACK(None)
//                 }
//             }
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for CRtype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         let value = match self.opcode {
//             opcodes::CompressedOpcode::C2 => {
//                 match self.funct1 {
//                     // C.JR / C.MV
//                     0 => match self.rs2 {
//                         0 => todo!("C.JR"),
//                         _ => todo!("C.MV"),
//                     },
//                     // C.EBREAK / C.JALR / C.ADD
//                     1 => match self.rs2 {
//                         0 => match self.rs1 {
//                             0 => todo!("C.EBREAK"),
//                             _ => todo!("C.JALR"),
//                         },
//                         _ => {
//                             // C.ADD
//                             let rs1v = _core.read_register(self.rs1);
//                             let rs2v = _core.read_register(self.rs2);
//                             Some(_core.bit_extend(rs1v.wrapping_add(rs2v) as i64) as u64)
//                         }
//                     },
//                     _ => panic!(),
//                 }
//             }
//             _ => panic!(),
//         };
//         if value.is_some() {
//             Stage::WRITEBACK(Some(WritebackStage {
//                 register: self.rs1,
//                 value: value.unwrap(),
//             }))
//         } else {
//             Stage::WRITEBACK(None)
//         }
//     }
// }

// impl ExecutionStage for CSStype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for CIWtype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for CLtype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for CStype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for CBtype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             _ => panic!(),
//         }
//     }
// }

// impl ExecutionStage for CJtype {
//     fn execute(&self, _core: &mut Core) -> Stage {
//         match self.opcode {
//             _ => panic!(),
//         }
//     }
// }
