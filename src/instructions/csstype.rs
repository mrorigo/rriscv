use crate::{
    cpu::{Core, Register},
    pipeline::Stage,
};

use super::{
    opcodes::CompressedOpcode, ImmediateDecoder, Instruction, InstructionExcecutor,
    InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CSStype {
    pub opcode: CompressedOpcode,
    pub uimm: u16,
    pub rs2: Register,
    pub funct3: u8,
}
impl ImmediateDecoder<u16, u16> for CSStype {
    fn decode_immediate(i: u16) -> u16 {
        let offset = ((i >> 7) & 0x38) | // offset[5:3] <= [12:10]
                        ((i >> 1) & 0x1c0); // offset[8:6] <= [9:7]
        let imm11_5 = (offset >> 5) & 0x3f;
        let imm4_0 = offset & 0x1f;
        (imm11_5 << 5) | (imm4_0)
    }
}

impl InstructionSelector<CSStype> for CSStype {}
impl InstructionExcecutor for Instruction<CSStype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
