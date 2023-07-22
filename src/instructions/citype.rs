use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::{Stage, WritebackStage},
};

use super::{
    functions::{C1_Funct3, Funct3},
    opcodes::CompressedOpcode,
    CompressedFormatDecoder, CompressedFormatType, ImmediateDecoder, Instruction,
    InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CItype {
    pub opcode: CompressedOpcode,
    pub rd: Register,
    pub imm: u16,
    pub funct3: Funct3,
}

impl CompressedFormatType for CItype {}

impl CompressedFormatDecoder<CItype> for CItype {
    fn decode(word: u16) -> CItype {
        CItype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rd: ((word >> 7) & 31) as u8,
            imm: CItype::decode_immediate(word as u16),
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

impl ImmediateDecoder<u16, u16> for CItype {
    fn decode_immediate(i: u16) -> u16 {
        let nzimm1612 = (i >> 2) & 31;
        let nzimm17 = (i >> 12) & 1;
        nzimm1612 | (nzimm17 << 5)
    }
}

#[allow(non_snake_case)]
impl Instruction<CItype> {
    fn C_LUI(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.LUI",
            args: Some(*args),
            funct: |core, args| {
                let value = (args.imm as u64) << 12;
                debug_trace!(println!(
                    "C.LUI x{}, {:#x?} ; x{} = {:#x?}",
                    args.rd, args.imm, args.rd, value
                ));
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value,
                }))
            },
        }
    }

    fn C_ADDI(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.ADDI",
            args: Some(*args),
            funct: |core, args| {
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: core
                        .read_register(args.rd)
                        .wrapping_add((args.imm as u64).sign_extend(64 - 12)),
                }))
            },
        }
    }

    pub fn C_LI(args: &CItype) -> Instruction<CItype> {
        Instruction {
            mnemonic: "C.LI",
            args: Some(*args),
            funct: |core, args| {
                let value = (args.imm as u64).sign_extend(64 - 6);
                Stage::WRITEBACK(Some(WritebackStage {
                    register: args.rd,
                    value: value,
                }))
            },
        }
    }
}

impl Display for Instruction<CItype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},{}", self.mnemonic, args.rd, args.imm,)
        }
    }
}

impl InstructionSelector<CItype> for CItype {
    fn select(&self, _xlen: Xlen) -> Instruction<CItype> {
        match self.opcode {
            CompressedOpcode::C1 => match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
                C1_Funct3::C_LUI => Instruction::C_LUI(self),
                C1_Funct3::C_ADDI => Instruction::C_ADDI(self),
                C1_Funct3::C_LI => Instruction::C_LI(self),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor for Instruction<CItype> {
    fn run(&self, core: &mut Core) -> Stage {
        debug_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
