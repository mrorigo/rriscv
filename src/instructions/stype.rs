use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::MajorOpcode, ImmediateDecoder, Instruction, InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Stype {
    pub opcode: MajorOpcode,
    pub rs1: Register,
    pub rs2: Register,
    pub imm12: u16,
    pub funct3: u8,
}

impl ImmediateDecoder<u32, u16> for Stype {
    fn decode_immediate(i: u32) -> u16 {
        let imm12 = (((i >> 7) & 0b11111) | ((i >> 20) & 0xffffe0)) as u16;
        let imm5 = ((i >> 7) & 31) as u16;
        imm12 | imm5 as u16
    }
}

impl InstructionSelector<Stype> for Stype {}

impl InstructionExcecutor for Instruction<Stype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
