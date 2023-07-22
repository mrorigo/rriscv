use std::fmt::Display;

use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::CompressedOpcode, CompressedFormatDecoder, CompressedFormatType, ImmediateDecoder,
    Instruction, InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CLtype {
    pub opcode: CompressedOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub imm: u16,
}

impl CompressedFormatType for CLtype {}

impl CompressedFormatDecoder<CLtype> for CLtype {
    fn decode(word: u16) -> CLtype {
        CLtype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rd: ((word >> 2) & 7) as Register + 8,
            rs1: ((word >> 7) & 31) as Register + 8,
            imm: CLtype::decode_immediate(word as u16),
        }
    }
}

impl ImmediateDecoder<u16, u16> for CLtype {
    fn decode_immediate(i: u16) -> u16 {
        ((i >> 7) & 0x38) | // offset[5:3] <= [12:10]
        ((i >> 4) & 0x4) | // offset[2] <= [6]
        ((i << 1) & 0x40) // offset[6] <= [5]
    }
}

impl Display for Instruction<CLtype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.args.is_some() {
            write!(f, "{}", self.mnemonic)
        } else {
            let args = self.args.unwrap();
            write!(
                f,
                "{} x{},x{},{}",
                self.mnemonic, args.rd, args.rs1, args.imm
            )
        }
    }
}

impl InstructionSelector<CLtype> for CLtype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<CLtype> {
        todo!()
    }
}
impl InstructionExcecutor for Instruction<CLtype> {
    fn run(&self, core: &mut Core) -> Stage {
        debug_trace!(println!("{}", self.to_string()));
        (self.funct)(core, &self.args.unwrap())
    }
}
