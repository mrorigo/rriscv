use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{opcodes::CompressedOpcode, Instruction, InstructionExcecutor, InstructionSelector};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CStype {
    pub opcode: CompressedOpcode,
    pub rs1: Register,
    pub rs2: Register,
    pub funct: u8,
    pub funct6: u8,
}

impl InstructionSelector<CStype> for CStype {}
impl InstructionExcecutor for Instruction<CStype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
