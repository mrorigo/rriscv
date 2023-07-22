use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{opcodes::CompressedOpcode, Instruction, InstructionExcecutor, InstructionSelector};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CRtype {
    pub opcode: CompressedOpcode,
    pub rs1: Register,
    pub rs2: Register,
    pub funct1: u8,
    pub funct3: u8,
}
impl InstructionSelector<CRtype> for CRtype {}
impl InstructionExcecutor for Instruction<CRtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
