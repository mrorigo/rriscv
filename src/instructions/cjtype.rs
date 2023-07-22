use crate::{cpu::Core, pipeline::Stage};

use super::{opcodes::CompressedOpcode, Instruction, InstructionExcecutor, InstructionSelector};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CJtype {
    pub opcode: CompressedOpcode,
    pub target: u16,
    pub funct3: u8,
}
impl InstructionSelector<CJtype> for CJtype {}
impl InstructionExcecutor for Instruction<CJtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
