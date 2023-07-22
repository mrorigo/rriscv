use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{opcodes::CompressedOpcode, Instruction, InstructionExcecutor, InstructionSelector};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CIWtype {
    pub opcode: CompressedOpcode,
    pub imm: u16,
    pub rd: Register,
    pub funct3: u8,
}
impl InstructionSelector<CIWtype> for CIWtype {}
impl InstructionExcecutor for Instruction<CIWtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
