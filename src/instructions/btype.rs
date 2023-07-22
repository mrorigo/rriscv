use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::MajorOpcode, ImmediateDecoder, Instruction, InstructionExcecutor, InstructionSelector,
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
