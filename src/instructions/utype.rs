use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::Stage,
};

use super::{
    opcodes::MajorOpcode, FormatDecoder, Instruction, InstructionExcecutor, InstructionFormatType,
    InstructionSelector, UncompressedFormatType,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Utype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub imm: u64,
}

impl InstructionFormatType for Utype {}
impl UncompressedFormatType for Utype {}

impl FormatDecoder<Utype> for Utype {
    fn decode(word: u32) -> Utype {
        Utype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            imm: (match word & 0x80000000 {
                0x80000000 => 0xffffffff00000000,
                _ => 0,
            } | ((word as u64) & 0xfffff000)) as u64,
        }
    }
}

impl Display for Instruction<Utype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},{}", self.mnemonic, args.rd, args.imm)
        }
    }
}

impl Instruction<Utype> {
    /// AUIPC forms a 32-bit offset from the 20-bit U-immediate, filling in the lowest 12 bits with
    /// zeros, adds this offset to the pc, then places the result in register rd
    #[allow(non_snake_case)]
    pub fn AUIPC(utype: &Utype) -> Instruction<Utype> {
        Instruction {
            mnemonic: "AUIPC",
            args: Some(*utype),
            funct: |core, args| {
                let value = core.prev_pc.wrapping_add((args.imm) as u64) as u32;
                Stage::writeback(args.rd, value as u64)
            },
        }
    }

    #[allow(non_snake_case)]
    pub fn LUI(utype: &Utype) -> Instruction<Utype> {
        Instruction {
            mnemonic: "LUI",
            args: Some(*utype),
            funct: |_core, args| Stage::writeback(args.rd, args.imm),
        }
    }
}

impl InstructionSelector<Utype> for Utype {
    fn select(&self, _xlen: Xlen) -> Instruction<Utype> {
        match self.opcode {
            MajorOpcode::AUIPC => Instruction::AUIPC(self),
            MajorOpcode::LUI => Instruction::LUI(self),
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor<Utype> for Instruction<Utype> {
    fn run(&self, core: &mut Core) -> Stage {
        instruction_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
