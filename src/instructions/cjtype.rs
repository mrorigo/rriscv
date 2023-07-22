use quark::Signs;

use crate::{cpu::Core, pipeline::Stage};

use super::{
    functions::C1_Funct3, opcodes::CompressedOpcode, CompressedFormatDecoder, CompressedFormatType,
    ImmediateDecoder, Instruction, InstructionExcecutor, InstructionSelector,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CJtype {
    pub opcode: CompressedOpcode,
    pub offset: u16,
    pub funct3: u8,
}

#[allow(non_snake_case)]
impl Instruction<CJtype> {
    /// C.J performs an unconditional control transfer. The [10-bit] offset is sign-extended and added
    /// to the pc to form the jump target address.
    pub fn C_J(args: &CJtype) -> Instruction<CJtype> {
        Instruction {
            args: Some(*args),
            mnemonic: "C.J",
            funct: |core, args| {
                let se_offs = (args.offset as i64).sign_extend(64 - 10) as i64;
                let target = (core.prev_pc as i64).wrapping_add(se_offs) as u64;
                instruction_trace!(println!(
                    "C.J: offs: {:#x?} => {:#x?} => {:#x?}",
                    args.offset, se_offs, target
                ));
                core.set_pc(target);
                Stage::WRITEBACK(None)
            },
        }
    }
}

impl CompressedFormatType for CJtype {}

impl CompressedFormatDecoder<CJtype> for CJtype {
    fn decode(word: u16) -> CJtype {
        CJtype {
            opcode: num::FromPrimitive::from_u8((word & 3) as u8).unwrap(),
            offset: CJtype::decode_immediate(word),
            funct3: (word >> 13) as u8,
        }
    }
}

impl ImmediateDecoder<u16, u16> for CJtype {
    fn decode_immediate(i: u16) -> u16 {
        // Have fun Mr Compiler, doing something efficient with this :^>
        (match i & 0x1000 {
            0x1000 => 0xf000,
            _ => 0,
        }) | ((i >> 1) & 0x800)
            | ((i >> 7) & 0x10)
            | ((i >> 1) & 0x300)
            | ((i << 2) & 0x400)
            | ((i >> 1) & 0x40)
            | ((i << 1) & 0x80)
            | ((i >> 2) & 0xe)
            | ((i << 3) & 0x20)
    }
}

impl InstructionSelector<CJtype> for CJtype {
    fn select(&self, _xlen: crate::cpu::Xlen) -> Instruction<CJtype> {
        match self.opcode {
            CompressedOpcode::C1 => match num::FromPrimitive::from_u8(self.funct3).unwrap() {
                C1_Funct3::C_J => Instruction::C_J(self),
                _ => todo!(),
            },
            _ => panic!(),
        }
    }
}
impl InstructionExcecutor for Instruction<CJtype> {
    fn run(&self, core: &mut Core) -> Stage {
        (self.funct)(core, &self.args.unwrap())
    }
}
