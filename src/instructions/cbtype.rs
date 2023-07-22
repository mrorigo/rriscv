use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::CompressedOpcode, ImmediateDecoder, Instruction, InstructionExcecutor,
    InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CBtype {
    pub opcode: CompressedOpcode,
    pub rs1: Register,
    pub offset: u16,
    pub funct3: u8,
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
