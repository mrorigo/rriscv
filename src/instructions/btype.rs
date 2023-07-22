use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::MajorOpcode, FormatDecoder, ImmediateDecoder, Instruction, InstructionExcecutor,
    InstructionFormatType, InstructionSelector,
};
//* 2.2 Base Instruction Formats */
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Btype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub rs2: Register,
    pub funct3: u8,
    pub imm12: u16,
}

impl InstructionFormatType for Btype {}

impl FormatDecoder<Btype> for Btype {
    fn decode(word: u32) -> Btype {
        Btype {
            opcode: num::FromPrimitive::from_u8((word & 0x7f) as u8).unwrap(),
            rd: ((word >> 7) & 31) as Register,
            rs1: ((word >> 15) & 31) as Register,
            rs2: ((word >> 20) & 31) as Register,
            imm12: Btype::decode_immediate(word),
            funct3: ((word >> 12) & 7) as u8,
        }
    }
}

impl ImmediateDecoder<u32, u16> for Btype {
    fn decode_immediate(i: u32) -> u16 {
        let imm12 = ((i >> 31) & 1) as u16;
        let imm105 = ((i >> 25) & 0b111111) as u16;
        let imm41 = ((i >> 8) & 0xf) as u16;
        let imm11 = ((i >> 7) & 1) as u16;
        (imm12 << 12) | (imm105 << 5) | (imm41 << 1) | (imm11 << 11)
    }
}

impl InstructionSelector<Btype> for Btype {}
impl InstructionExcecutor for Instruction<Btype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
