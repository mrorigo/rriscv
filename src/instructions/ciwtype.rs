use std::fmt::Display;

use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    functions::{C0_Funct3, Funct3},
    opcodes::CompressedOpcode,
    CompressedFormatDecoder, CompressedFormatType, Instruction, InstructionExcecutor,
    InstructionFormatType, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CIWtype {
    pub opcode: CompressedOpcode,
    pub imm: u16,
    pub rd: Register,
    pub funct3: Funct3,
}

impl InstructionFormatType for CIWtype {}
impl CompressedFormatType for CIWtype {}

impl CompressedFormatDecoder<CIWtype> for CIWtype {
    fn decode(word: u16) -> CIWtype {
        CIWtype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            imm: ((word >> 5) as u16) & ((1 << 7) - 1),
            rd: ((word >> 2) & 7) as Register + 8,
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

impl Display for Instruction<CIWtype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(f, "{} x{},x2,{}", self.mnemonic, args.rd, args.imm)
        }
    }
}

#[allow(non_snake_case)]
impl Instruction<CIWtype> {
    // adds a zero-extended non-zero immediate, scaled by 4, to the stack pointer, x2, and writes the result to rd
    pub fn C_ADDI4SPN(ciwtype: &CIWtype) -> Instruction<CIWtype> {
        Instruction {
            args: Some(*ciwtype),
            mnemonic: "C.ADDI4SPN",
            funct: |core, args| {
                let sp = core.read_register(2);
                let value = sp + (args.imm as u64 >> 2);
                debug_assert!(args.imm != 0);
                Stage::writeback(args.rd, value)
            },
        }
    }
}

impl InstructionSelector<CIWtype> for CIWtype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<CIWtype> {
        match num::FromPrimitive::from_u8(self.funct3 as u8).unwrap() {
            C0_Funct3::C_ADDI4SPN => Instruction::C_ADDI4SPN(self),
            _ => panic!(),
        }
    }
}

impl InstructionExcecutor<CIWtype> for Instruction<CIWtype> {
    fn run(&self, core: &mut Core) -> Stage {
        instruction_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
