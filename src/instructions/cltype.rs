use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::CompressedOpcode, ImmediateDecoder, Instruction, InstructionExcecutor,
    InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CLtype {
    pub opcode: CompressedOpcode,
    pub rd: Register,
    pub rs1: Register,
    pub imm: u16,
}

impl ImmediateDecoder<u16, u16> for CLtype {
    fn decode_immediate(i: u16) -> u16 {
        ((i >> 7) & 0x38) | // offset[5:3] <= [12:10]
        ((i >> 4) & 0x4) | // offset[2] <= [6]
        ((i << 1) & 0x40) // offset[6] <= [5]
    }
}

impl InstructionSelector<CLtype> for CLtype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<CLtype> {
        todo!()
    }
}
impl InstructionExcecutor for Instruction<CLtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
