use std::fmt::Display;

use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{Stage, WritebackStage},
};

use super::{
    functions::{Funct3, Funct7, Op_Funct3, RV32M_Funct3},
    opcodes::MajorOpcode,
    FormatDecoder, Instruction, InstructionExcecutor, InstructionFormat, InstructionFormatType,
    InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rtype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub rs2: Register,
    pub funct3: Funct3,
    pub funct7: Funct7,
}

impl InstructionFormatType for Rtype {}

impl FormatDecoder<Rtype> for Rtype {
    fn decode(word: u32) -> Rtype {
        Rtype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            rs1: ((word >> 15) & 31) as Register,
            rs2: ((word >> 20) & 31) as Register,
            funct3: num::FromPrimitive::from_u8(((word >> 12) & 7) as u8).unwrap(),
            funct7: num::FromPrimitive::from_u8(((word >> 25) & 0x7f) as u8).unwrap(),
        }
    }
}

impl Display for Instruction<Rtype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(
                f,
                "{} x{},x{},x{}",
                self.mnemonic, args.rd, args.rs1, args.rs2
            )
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<Rtype> {
    pub fn AND(itype: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"AND",
            args: Some(*itype),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = core.bit_extend((r1v & r2v) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn OR(itype: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"OR",
            args: Some(*itype),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = core.bit_extend((r1v | r2v) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn MUL(itype: &Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"MUL",
            args: Some(*itype),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = core.bit_extend(r1v.wrapping_mul(r2v) as i64) as u64;
                Stage::writeback(args.rd, value)
            },
        }
    }

    pub fn MULH(itype: Rtype) -> Instruction<Rtype> {
        Instruction {
            mnemonic: &"MULH",
            args: Some(itype),
            funct: |core, args| {
                let r1v = core.read_register(args.rs1);
                let r2v = core.read_register(args.rs2);
                let value = match core.xlen {
                    Xlen::Bits32 => core.bit_extend((r1v as i64 * r2v as i64) >> 32) as u64,
                    Xlen::Bits64 => ((r1v as i128) * (r2v as i128) >> 64) as u64,
                };
                Stage::writeback(args.rd, value)
            },
        }
    }
}

impl InstructionSelector<Rtype> for Rtype {
    fn select(&self, _xlen: Xlen) -> Instruction<Rtype> {
        match self.opcode {
            MajorOpcode::OP => match self.funct7 {
                // RV32M
                Funct7::RV32M => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                    RV32M_Funct3::MUL => Instruction::MUL(self),
                    _ => panic!(),
                },
                Funct7::B0100000 => todo!("SUB&SRA"),
                Funct7::B0000000 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                    Op_Funct3::ADD_SUB => todo!("ADD, since Funct7=0"),
                    Op_Funct3::SRL_SRA => todo!("SRL, since Funct7=0"),
                    Op_Funct3::AND => Instruction::AND(self),
                    Op_Funct3::OR => Instruction::OR(self),
                    _ => todo!(),
                },
                _ => todo!("R-type Funct7 not supported"),
            },
            _ => panic!(),
        }
    }
}
impl InstructionExcecutor for Instruction<Rtype> {
    fn run(&self, core: &mut Core) -> Stage {
        instruction_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
