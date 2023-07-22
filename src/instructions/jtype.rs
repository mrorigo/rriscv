use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::MajorOpcode, ImmediateDecoder, Instruction, InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Jtype {
    pub opcode: MajorOpcode,
    pub rd: Register,
    pub imm20: u32,
}

impl ImmediateDecoder<u32, u32> for Jtype {
    fn decode_immediate(i: u32) -> u32 {
        let imm20 = ((i >> 31) & 0b1) as u32;
        let imm101 = ((i >> 21) & 0b1111111111) as u32;
        let imm11 = ((i >> 20) & 0b1) as u32;
        let imm1912 = ((i >> 12) & 0b11111111) as u32;

        let imm = (imm20 << 20) | (imm101 << 1) | (imm11 << 11) | (imm1912 << 12);
        ((imm) << 11) >> 12
    }
}

impl InstructionSelector<Jtype> for Jtype {}

impl InstructionExcecutor for Instruction<Jtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
