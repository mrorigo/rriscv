use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::CompressedOpcode, CompressedFormatDecoder, CompressedFormatType, ImmediateDecoder,
    Instruction, InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CBtype {
    pub opcode: CompressedOpcode,
    pub rs1: Register,
    pub offset: u16,
    pub funct3: u8,
}

impl CompressedFormatType for CBtype {}

impl CompressedFormatDecoder<CBtype> for CBtype {
    fn decode(word: u16) -> CBtype {
        CBtype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            rs1: 8 + ((word >> 7) & 3) as u8,
            offset: CBtype::decode_immediate(word as u16),
            funct3: num::FromPrimitive::from_u8(((word >> 13) & 0x7) as u8).unwrap(),
        }
    }
}

impl ImmediateDecoder<u16, u16> for CBtype {
    fn decode_immediate(_i: u16) -> u16 {
        todo!()
    }
}

impl InstructionSelector<CBtype> for CBtype {}
impl InstructionExcecutor for Instruction<CBtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
