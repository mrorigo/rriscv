use std::fmt::Display;

use crate::{
    cpu::{Register, Xlen},
    pipeline::Stage,
};

use super::{
    functions::Funct3, opcodes::CompressedOpcode, CompressedFormatDecoder, CompressedFormatType,
    Instruction, InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CRtype {
    pub opcode: CompressedOpcode,
    pub rs1_rd: Register,
    pub rs2: Register,
    pub funct1: u8,
    pub funct3: Funct3,
}

impl InstructionFormatType for CRtype {}
impl CompressedFormatType for CRtype {}

impl CompressedFormatDecoder<CRtype> for CRtype {
    fn decode(word: u16) -> CRtype {
        CRtype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rs2: ((word >> 2) & 31) as Register,
            rs1_rd: (word >> 7 & 31) as Register,
            funct1: (word >> 12) as u8 & 1,
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<CRtype> {
    pub fn C_ADD(crtype: &CRtype) -> Instruction<CRtype> {
        Instruction {
            args: Some(*crtype),
            mnemonic: "C.ADD",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let rs2v = core.read_register(args.rs2);
                let value = core.bit_extend(rs1v.wrapping_add(rs2v) as i64) as u64;
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }

    pub fn C_EBREAK(args: &CRtype) -> Instruction<CRtype> {
        Instruction {
            mnemonic: &"C.EBREAK",
            args: Some(*args),
            funct: |_core, _args| Stage::TRAP(crate::cpu::TrapCause::Breakpoint),
        }
    }

    pub fn C_JALR(args: &CRtype) -> Instruction<CRtype> {
        Instruction {
            mnemonic: &"C.JALR",
            args: Some(*args),
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                let old_pc = core.pc();
                core.set_pc(rs1v);
                Stage::writeback(1, old_pc)
            },
        }
    }

    pub fn C_JR(crtype: &CRtype) -> Instruction<CRtype> {
        Instruction {
            args: Some(*crtype),
            mnemonic: "C.JR",
            funct: |core, args| {
                let rs1v = core.read_register(args.rs1_rd);
                core.set_pc(rs1v);
                Stage::WRITEBACK(None)
            },
        }
    }

    pub fn C_MV(crtype: &CRtype) -> Instruction<CRtype> {
        Instruction {
            args: Some(*crtype),
            mnemonic: "C.MV",
            funct: |core, args| {
                let value = core.read_register(args.rs2);
                Stage::writeback(args.rs1_rd, value)
            },
        }
    }
}

impl Display for Instruction<CRtype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},x{}", self.mnemonic, args.rs1_rd, args.rs2)
        }
    }
}

impl InstructionSelector<CRtype> for CRtype {
    fn select(&self, _xlen: Xlen) -> Instruction<CRtype> {
        match self.opcode {
            CompressedOpcode::C2 => {
                match self.funct1 {
                    // C.JR / C.MV
                    0 => match self.rs2 {
                        0 => Instruction::C_JR(self),
                        _ => Instruction::C_MV(self),
                    },
                    // C.EBREAK / C.JALR / C.ADD
                    1 => match self.rs2 {
                        0 => match self.rs1_rd {
                            0 => Instruction::C_EBREAK(self), //todo!("C.EBREAK"),
                            _ => Instruction::C_JALR(self),
                        },
                        _ => Instruction::C_ADD(self),
                    },
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }
}
