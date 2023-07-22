use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{Stage, WritebackStage},
};

use super::{
    functions::{CSR_Funct3, Load_Funct3, OpImm32_Funct3, OpImm_Funct3},
    opcodes::MajorOpcode,
    FormatDecoder, Instruction, InstructionExcecutor, InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Itype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub funct3: u8,
    pub imm12: u16,
}

impl InstructionFormatType for Itype {}

impl FormatDecoder<Itype> for Itype {
    fn decode(word: u32) -> Itype {
        Itype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            rs1: ((word >> 15) & 31) as Register,
            imm12: (word >> 20) as u16,
            funct3: ((word >> 12) & 7) as u8,
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Itype> {
    pub fn CSRRW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"CSRRW",
            args: Some(*args),
            funct: |core, args| {
                let csr_register = num::FromPrimitive::from_u16(args.imm12).unwrap();
                // "If rd=x0, then the instruction shall not read the CSR"
                let value = if args.rd != 0 {
                    Some(core.read_csr(csr_register))
                } else {
                    None
                };

                let rs1v = core.read_register(args.rs1);
                core.write_csr(csr_register, rs1v);

                if value.is_none() {
                    Stage::WRITEBACK(None)
                } else {
                    Stage::writeback(args.rd, value.unwrap())
                }
            },
        }
    }

    pub fn CSRRS(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"CSRRS",
            args: Some(*args),
            funct: |core, args| {
                let csr_register = num::FromPrimitive::from_u16(args.imm12).unwrap();
                let value = core.read_csr(csr_register);
                if args.rs1 != 0 {
                    let rs1v = core.read_register(args.rs1);
                    core.write_csr(csr_register, value | rs1v);
                }
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn ADDI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ADDI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend(rs1v.wrapping_add(seimm as i64)) as u64;
                // debug_trace!(println!(
                //     "ADDI x{}, x{}, {}  ; x{} = {:#x?}",
                //     args.rd, args.rs1, seimm as i64, args.rd, value
                // ));
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn ADDIW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ADDIW",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend(rs1v.wrapping_add(seimm as i64)) as i32 as u64;
                debug_trace!(println!(
                    "ADDIW x{}, x{}, {}  ; x{} = {:#x?}",
                    args.rd, args.rs1, seimm as i64, args.rd, value
                ));
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn ORI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"ORI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1) as i64;
                let seimm = (args.imm12 as u64).sign_extend(64 - 12);
                let value = core.bit_extend(rs1v | seimm as i64) as u64;
                debug_trace!(println!(
                    "ORI x{}, x{}, {:#x?}  ; x{} = {:#x?}",
                    args.rd, args.rs1, seimm as u64, args.rd, value
                ));
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn JALR(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"JALR",
            args: Some(*args),
            funct: |core, args| {
                let value = core.pc;

                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;

                let rs1v = core.read_register(args.rs1);
                let target = (rs1v as i64 + se_imm12) as u64;
                core.set_pc(target);

                if args.rd != 0 {
                    Stage::writeback(args.rd, value)
                } else {
                    Stage::WRITEBACK(None)
                }
            },
        }
    }

    pub fn SLLI(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SLLI",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let mask = match core.xlen {
                    Xlen::Bits32 => 0x1f,
                    Xlen::Bits64 => 0x3f,
                };
                let shamt = (args.imm12) & mask;
                let value = ((rs1v as i64) << shamt) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn SLLIW(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"SLLIW",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1);
                // the shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I.
                let shamt = args.imm12 & 0b111111;
                let value = ((rs1v as i64) << shamt) as i32 as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn LD(args: &Itype) -> Instruction<Itype> {
        Instruction {
            mnemonic: &"LD",
            args: Some(*args),
            funct: |core, args| {
                let se_imm12 = (args.imm12 as u64).sign_extend(64 - 12) as i64;
                let rs1v = core.read_register(args.rs1);
                let addr = (rs1v as i64 + se_imm12) as u64;
                debug_trace!(println!(
                    "LD: rs1v={:#x?} se_imm12={:#x?} addr={:#x?}",
                    rs1v, se_imm12, addr
                ));
                Stage::MEMORY(crate::pipeline::MemoryAccess::READ64(addr, args.rd))
            },
        }
    }
}

impl Display for Instruction<Itype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(
                f,
                "{} x{},x{},{}",
                self.mnemonic, args.rd, args.rs1, args.imm12,
            )
        }
    }
}

impl InstructionSelector<Itype> for Itype {
    fn select(&self, _xlen: Xlen) -> Instruction<Itype> {
        match self.opcode {
            MajorOpcode::OP_IMM => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                OpImm_Funct3::ADDI => Instruction::ADDI(self),
                OpImm_Funct3::ORI => Instruction::ORI(self),
                OpImm_Funct3::SLLI => Instruction::SLLI(self),
                _ => panic!(),
            },
            MajorOpcode::SYSTEM => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                CSR_Funct3::CSRRS => Instruction::CSRRS(self),
                CSR_Funct3::CSRRW => Instruction::CSRRW(self),
                _ => panic!(),
            },
            MajorOpcode::JALR => Instruction::JALR(self),
            MajorOpcode::OP_IMM_32 => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                OpImm32_Funct3::ADDIW => Instruction::ADDIW(self),
                OpImm32_Funct3::SLLIW => Instruction::SLLIW(self),
                _ => panic!(),
            },
            MajorOpcode::LOAD => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                Load_Funct3::LD => Instruction::LD(self),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<Itype> {
    fn run(&self, core: &mut Core) -> Stage {
        debug_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
