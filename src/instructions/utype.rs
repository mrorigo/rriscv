use std::fmt::Display;

use quark::Signs;

use crate::{
    cpu::{Core, Register, Xlen},
    pipeline::Stage,
};

use super::{
    opcodes::MajorOpcode, FormatDecoder, Instruction, InstructionExcecutor, InstructionFormatType,
    InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Utype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub imm20: u32,
}

impl InstructionFormatType for Utype {}

impl FormatDecoder<Utype> for Utype {
    fn decode(word: u32) -> Utype {
        Utype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            imm20: (word >> 12),
        }
    }
}

impl Display for Instruction<Utype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},{}", self.mnemonic, args.rd, args.imm20)
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
                let se_imm20 = args.imm20.sign_extend(32 - 20);
                // const M: u32 = 1 << (20 - 1);
                // let se_imm20 = (args.imm20 ^ M) - M;
                let value = core.prev_pc.wrapping_add((se_imm20 << 12) as u64) as u32;
                // instruction_trace!(println!(
                //     "AUIPC x{}, {:#x?}\t; pc={:#x?} x{}={:#x?}",
                //     args.rd, se_imm20, core.prev_pc, args.rd, value
                // ));
                Stage::writeback(args.rd, value as u64)
            },
        }
    }

    #[allow(non_snake_case)]
    pub fn LUI(utype: &Utype) -> Instruction<Utype> {
        Instruction {
            mnemonic: "LUI",
            args: Some(*utype),
            funct: |_core, args| {
                let value = (args.imm20 as u64) << 12;
                Stage::writeback(args.rd, value)
            },
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

impl InstructionExcecutor for Instruction<Utype> {
    fn run(&self, core: &mut Core) -> Stage {
        instruction_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
